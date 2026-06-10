//! Runtime script bindings via embedded **Rhai** (see `docs/script_bindings_plan.md`).
//!
//! Feature-gated behind `runtime-bindings` (ON by default for the dev/test
//! profile and the documented distribution build — downstream single-binary
//! consumers customize without recompiling; `script-bindings` is a back-compat
//! alias). The ONLY module in
//! the workspace that embeds Rhai; core/engine/package are untouched. Pure-Rust,
//! no FFI, no ABI — the reliable successor to the abandoned libperl approach.
//!
//! ## Model (the "fine seam", Model A)
//!
//! A binding is a Rhai script that calls registration functions (`DefMacro`,
//! `DefConstructor`, …) with the binding body as a Rhai function value. Each
//! registration installs a **native** latexml-oxide `Definition` whose body
//! closure trampolines into the Rhai engine. Prototype parsing, argument
//! reading, the gullet/stomach/document machinery all stay native; Rhai runs
//! only the body.
//!
//! ## Seams implemented
//!
//! * **`DefMacro`** (expandable) — body receives args as strings, returns a string of TeX,
//!   re-tokenized via `mouth::tokenize_internal` so an expansion to `\textit{x}` faithfully yields
//!   a control-sequence token.
//! * **`DefConstructor`** (construction) — two forms. A string XML template, or an imperative body
//!   that reads like the Perl original: it receives a `Document` **proxy** as its first argument
//!   (Perl's `$_[0]`) and each digested argument as an opaque handle, e.g. `|document, x| {
//!   document.openElement("ltx:emph"); document.absorb(x); document.closeElement("ltx:emph"); }`.
//!   The proxy's methods resolve a thread-local **active context** (the live `&mut Document` +
//!   props, published for the duration of the call; never borrowed across a re-entrant call).
//!
//! ## Lifecycle
//!
//! `load_script` compiles the script to an `AST`, runs it once to collect the
//! registrations, wraps engine + AST in `Rc`, and installs one native definition
//! per registration — each capturing `Rc<Engine>`/`Rc<AST>` so a body stays
//! callable for the whole conversion (a deferred `FnPtr::call` needs both alive).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rhai::{AST, Dynamic, Engine, EvalAltResult, FnPtr, Map};

use latexml_core::BoxOps;
use latexml_core::binding::def::builder::{ConstructorBuilder, EnvironmentBuilder, OptionValue};
use latexml_core::binding::def::dialect::{def_conditional, def_macro, def_primitive};
use latexml_core::binding::def::replacement;
use latexml_core::common::arena::{self, SymHashMap};
use latexml_core::common::def_parser::parse_prototype;
use latexml_core::common::dimension::Dimension;
use latexml_core::common::error::{Error, Result};
use latexml_core::common::number::Number;
use latexml_core::common::numeric_ops::NumericOps;
use latexml_core::common::object::Object;
use latexml_core::common::store::Stored;
use latexml_core::definition::ConditionalClosure;
use latexml_core::definition::argument::ArgWrap;
use latexml_core::definition::conditional::ConditionalOptions;
use latexml_core::definition::expandable::ExpandableOptions;
use latexml_core::definition::math_primitive::MathPrimitiveOptions;
use latexml_core::definition::primitive::PrimitiveOptions;
use latexml_core::definition::{
  BeforeDigestClosure, ConstructionClosure, DigestionClosure, ExpansionBody, ExpansionClosure,
  FontDirective, PrimitiveBody, PrimitiveClosure, PropertiesClosure, ReplacementClosure, Reversion,
};
use latexml_core::digested::Digested;
use latexml_core::document::Document;
use latexml_core::mouth;
use latexml_core::state::Scope;
use latexml_core::token::{Catcode, Token};
use latexml_core::tokens::Tokens;
use latexml_core::whatsit::Whatsit;
// `Error!` expands a `Fatal!`/`fatal!` arm (too-many-errors escalation); the
// whole macro chain must be in scope at the expansion site (non-hygienic).
#[allow(unused_imports)]
use latexml_core::{Fatal, fatal};

// Sandbox limits (docs/script_bindings_plan.md §6).
const MAX_OPERATIONS: u64 = 50_000_000;
const MAX_CALL_LEVELS: usize = 128;
const MAX_STRING_SIZE: usize = 4 * 1024 * 1024;

/// A constructor's replacement — either an XML template or an imperative body.
#[derive(Clone)]
enum ConstructorRepl {
  Template(String),
  Closure(FnPtr),
}

/// A compiled script, cached by source so the (relatively expensive) Rhai
/// compile happens once per unique binding. The RUN is per-load (per
/// conversion) — Perl semantics: each `Def…`/side-effect call installs into
/// the current State sequentially as the script executes.
#[derive(Clone)]
struct CachedScript {
  engine: Rc<Engine>,
  ast:    Rc<AST>,
}

thread_local! {
  /// The script currently being RUN (a stack — a script can trigger loading
  /// another script). `Def…` registration closures grab these handles to wire
  /// their trampolines immediately, in script order — so e.g. a
  /// `DeclareOption` is installed before a following `ProcessOptions()` runs.
  static CURRENT_SCRIPT: RefCell<Vec<(Rc<Engine>, Rc<AST>)>> = const { RefCell::new(Vec::new()) };

  /// Definitions installed by the innermost running script (the load_script
  /// return value).
  static WIRED_COUNT: RefCell<usize> = const { RefCell::new(0) };

  static SCRIPT_CACHE: RefCell<HashMap<String, CachedScript>> =
    RefCell::new(HashMap::new());

  /// Active-context stack for constructor bodies. Each entry publishes the live
  /// `&mut Document` + props of an in-flight `DefConstructor` call as raw
  /// pointers, so the `Document`-proxy methods can reach them. A stack (not a
  /// single slot) so a constructor body that triggers nested construction works.
  /// (Digested args are passed to the body directly as handles, not via here.)
  static CTOR_CTX: RefCell<Vec<CtorCtx>> = const { RefCell::new(Vec::new()) };

  /// Active-context stack for whatsit-receiving hook closures (`afterDigest`, …).
  /// Lets a parameterless hook body reach the in-flight whatsit via `whatsit()` —
  /// referenced only when needed (the "omit as implied" model).
  /// Entries are (whatsit pointer, mutable?). Digestion hooks (`afterDigest`,
  /// `properties`-adjacent) publish mutable whatsits; construction hooks
  /// (`before/afterConstruct`, `reversion`, `sizer`) publish READ-ONLY ones —
  /// `setProperty` errors there instead of mutating through a shared ref.
  static WHATSIT_CTX: RefCell<Vec<(*mut Whatsit, bool)>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy)]
struct CtorCtx {
  document: *mut Document,
  props:    *const SymHashMap<Stored>,
}

/// The engine/AST of the innermost running script (for immediate wiring).
fn current_script() -> std::result::Result<(Rc<Engine>, Rc<AST>), Box<EvalAltResult>> {
  CURRENT_SCRIPT.with(|c| {
    c.borrow()
      .last()
      .cloned()
      .ok_or_else(|| Box::<EvalAltResult>::from("registration called outside a script load"))
  })
}

/// Count one installed definition (the `load_script` return value).
fn note_wired() { WIRED_COUNT.with(|c| *c.borrow_mut() += 1); }

/// Wire immediately from inside a registration closure: resolve the current
/// script handles, run the wiring fn, count, map errors to Rhai.
fn wire_now(
  wire: impl FnOnce(&Rc<Engine>, &Rc<AST>) -> Result<()>,
) -> std::result::Result<(), Box<EvalAltResult>> {
  let (engine, ast) = current_script()?;
  wire(&engine, &ast).map_err(rhai_err)?;
  note_wired();
  Ok(())
}

/// Copy the top active-context out (so we never hold the `CTOR_CTX` borrow
/// across a Document call that might re-enter the bridge).
fn current_ctx() -> std::result::Result<CtorCtx, Box<EvalAltResult>> {
  CTOR_CTX.with(|c| {
    c.borrow()
      .last()
      .copied()
      .ok_or_else(|| Box::<EvalAltResult>::from("document op called outside a constructor body"))
  })
}

/// Rhai proxy for the live document, passed to a constructor body as its first
/// argument — so a binding reads like the Perl original (`$document->method`).
/// It carries no pointer itself; its methods resolve the active-context, so it
/// is only valid inside a constructor body (a method call outside one is a clean
/// error, never UB).
#[derive(Clone)]
struct DocProxy;

/// Rhai proxy for the in-flight whatsit, obtained inside a hook body via
/// `whatsit()`. Like `DocProxy`, it carries no pointer; methods resolve the
/// active whatsit context.
#[derive(Clone)]
struct WhatsitProxy;

/// Resolve the top whatsit active-context (used by `WhatsitProxy` methods).
fn current_whatsit() -> std::result::Result<*mut Whatsit, Box<EvalAltResult>> {
  current_whatsit_entry().map(|(w, _)| w)
}

/// The top whatsit entry incl. its mutability flag.
fn current_whatsit_entry() -> std::result::Result<(*mut Whatsit, bool), Box<EvalAltResult>> {
  WHATSIT_CTX.with(|c| {
    c.borrow()
      .last()
      .copied()
      .ok_or_else(|| Box::<EvalAltResult>::from("whatsit() called outside a hook body"))
  })
}

mod engine;
mod wire;
use engine::make_engine;
#[allow(unused_imports)]
use wire::*;

/// Load a binding script from a file on disk (e.g. `mypkg.sty.rhai`). Thin
/// wrapper over [`load_script`]; reading user-supplied files from a user-named
/// path is in scope (like reading `.sty` from texmf).
pub fn load_file(path: &str) -> Result<usize> {
  let src = std::fs::read_to_string(path)?;
  load_script(&src)
}

/// Load a binding script: compile once (cached by source), then RUN the script
/// against the current State. Every `Def…` registration wires its native
/// definition IMMEDIATELY as the script executes (sequential, Perl `.ltxml`
/// semantics — so `DeclareOption` precedes a later `ProcessOptions()`), and
/// top-level side-effect calls (`RawTeX`, `Let`, `NewCounter`, `AssignValue`,
/// `DefRegister`, …) re-apply on every load — i.e. per conversion, not just
/// the first one that happened to compile the script. Returns the number of
/// definitions installed.
pub fn load_script(src: &str) -> Result<usize> {
  // Fast path: a previously-compiled script with the same source.
  let cached = SCRIPT_CACHE.with(|c| c.borrow().get(src).cloned());
  let cached = match cached {
    Some(c) => c,
    None => {
      let engine = make_engine();
      let ast = engine
        .compile(src)
        .map_err(|e| Error::from(format!("script-binding compile error: {e}")))?;
      let cs = CachedScript {
        engine: Rc::new(engine),
        ast:    Rc::new(ast),
      };
      SCRIPT_CACHE.with(|c| {
        c.borrow_mut().insert(src.to_string(), cs.clone());
      });
      cs
    },
  };

  // Publish the script handles for the registration closures, run, unpublish
  // (a stack, so a script loading another script nests correctly).
  CURRENT_SCRIPT.with(|c| {
    c.borrow_mut()
      .push((cached.engine.clone(), cached.ast.clone()))
  });
  let before = WIRED_COUNT.with(|c| *c.borrow());
  let run = cached
    .engine
    .run_ast(&cached.ast)
    .map_err(|e| Error::from(format!("script-binding run error: {e}")));
  CURRENT_SCRIPT.with(|c| {
    c.borrow_mut().pop();
  });
  run?;
  Ok(WIRED_COUNT.with(|c| *c.borrow()) - before)
}

/// Build a partial `Font` from a Rhai map (family/series/shape/… keys — the
/// `fontmap!` analog, shared by `MergeFont` and the `font =>` option).
fn font_from_rhai_map(opts: Map) -> latexml_core::common::font::Font {
  let mut font = latexml_core::common::font::Font::default();
  for (key, val) in opts {
    let v = dynamic_to_string(val.clone());
    match key.as_str() {
      "family" => font.family = Some(v.into()),
      "series" => font.series = Some(v.into()),
      "shape" => font.shape = Some(v.into()),
      "encoding" => font.encoding = Some(v.into()),
      "language" => font.language = Some(v.into()),
      "mathstyle" => font.mathstyle = Some(v.into()),
      "size" => {
        font.size = val
          .as_float()
          .ok()
          .or_else(|| val.as_int().ok().map(|i| i as f64))
      },
      _ => {},
    }
  }
  font
}

/// Convert a Rhai object map into a whatsit property map. `Digested` handles
/// (from `DigestText`, `RefStepCounter`, …) pass through as `Stored::Digested`;
/// a `Digested` under the `font` key contributes its FONT (the IEEEproof
/// "titlefont" idiom); everything else lands as its string form.
fn rhai_map_to_props(map: Map) -> SymHashMap<Stored> {
  let mut props: SymHashMap<Stored> = SymHashMap::default();
  for (k, v) in map {
    if let Some(d) = v.clone().try_cast::<Digested>() {
      if k.as_str() == "font" {
        if let Ok(Some(f)) = d.get_font() {
          props.insert("font", Stored::Font(Rc::new(f.into_owned())));
          continue;
        }
      }
      props.insert(k.as_str(), Stored::Digested(d));
    } else {
      props.insert(k.as_str(), dynamic_to_string(v).into());
    }
  }
  props
}

/// Map a latexml error into a Rhai error (the standard boundary conversion).
fn rhai_err(e: Error) -> Box<EvalAltResult> { Box::<EvalAltResult>::from(e.to_string()) }

/// Parse a scope string ("local"/"global", anything else = None → TeX default).
fn scope_of(scope: &str) -> Option<Scope> {
  match scope {
    "local" => Some(Scope::Local),
    "global" => Some(Scope::Global),
    _ => None,
  }
}

/// Marshal a `Stored` out to a Rhai value: `Digested` as a live handle,
/// scalars as scalars, the rest via `Display`.
fn stored_to_dynamic(v: Stored) -> Dynamic {
  match v {
    Stored::Digested(d) => Dynamic::from(d),
    Stored::String(s) => Dynamic::from(arena::to_string(s)),
    Stored::Bool(b) => Dynamic::from(b),
    other => Dynamic::from(other.to_string()),
  }
}

/// Marshal a digested macro argument into a Rhai value. Every argument is passed
/// as its TeX-source string (authors parse it in-script as needed).
fn arg_to_dynamic(arg: ArgWrap) -> Dynamic { Dynamic::from(arg.to_string()) }

/// Read a Rhai return value back as a string (the TeX-source of the expansion).
fn dynamic_to_string(d: Dynamic) -> String {
  if d.is_string() {
    d.into_string().unwrap_or_default()
  } else {
    d.to_string()
  }
}

/// Split a keyval dict's TeX-source form ("k=v, k2={v 2}") into (key, value)
/// pairs — comma/equals splitting at brace depth 0 only, one level of outer
/// braces stripped from values (keyval semantics).
fn keyval_pairs(kv: &str) -> Vec<(String, String)> {
  let mut pairs = Vec::new();
  let mut depth = 0usize;
  let mut item = String::new();
  let mut items: Vec<String> = Vec::new();
  for c in kv.chars() {
    match c {
      '{' => {
        depth += 1;
        item.push(c);
      },
      '}' => {
        depth = depth.saturating_sub(1);
        item.push(c);
      },
      ',' if depth == 0 => items.push(std::mem::take(&mut item)),
      _ => item.push(c),
    }
  }
  if !item.trim().is_empty() {
    items.push(item);
  }
  for it in items {
    let (k, v) = match it.find('=') {
      Some(eq) => (it[..eq].trim(), it[eq + 1..].trim()),
      None => (it.trim(), ""),
    };
    if k.is_empty() {
      continue;
    }
    let v = v
      .strip_prefix('{')
      .and_then(|s| s.strip_suffix('}'))
      .unwrap_or(v);
    pairs.push((k.to_string(), v.to_string()));
  }
  pairs
}

#[cfg(test)]
mod tests;
