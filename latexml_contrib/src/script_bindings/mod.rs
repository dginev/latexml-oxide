//! Runtime script bindings via embedded **Rhai** (see `docs/parity/script_bindings_plan.md`).
//!
//! Feature-gated behind `runtime-bindings` (ON by default and in the distribution
//! build — downstream single-binary consumers customize without recompiling; the
//! module keeps its original name, the feature does not). The ONLY module in the
//! workspace that embeds Rhai; core/engine/package are untouched. Pure-Rust, no
//! FFI, no ABI — the reliable successor to the abandoned libperl approach.
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

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use latexml_core::{
  BoxOps,
  binding::def::{
    builder::{ConstructorBuilder, EnvironmentBuilder, OptionValue},
    dialect::{def_conditional, def_macro, def_primitive},
    replacement,
  },
  common::{
    arena::{self, SymHashMap},
    def_parser::parse_prototype,
    dimension::Dimension,
    error::{Error, Result},
    number::Number,
    numeric_ops::NumericOps,
    object::Object,
    store::Stored,
  },
  definition::{
    BeforeDigestClosure, ConditionalClosure, ConstructionClosure, DigestionClosure, ExpansionBody,
    ExpansionClosure, FontDirective, PrimitiveBody, PrimitiveClosure, PropertiesClosure,
    ReplacementClosure, Reversion, argument::ArgWrap, conditional::ConditionalOptions,
    expandable::ExpandableOptions, math_primitive::MathPrimitiveOptions,
    primitive::PrimitiveOptions,
  },
  digested::Digested,
  document::Document,
  mouth,
  state::Scope,
  token::{Catcode, Token},
  tokens::Tokens,
  whatsit::Whatsit,
};
// `Error!` expands a `Fatal!`/`fatal!` arm (too-many-errors escalation); the
// whole macro chain must be in scope at the expansion site (non-hygienic).
#[allow(unused_imports)]
use latexml_core::{Fatal, fatal};
use rhai::{AST, Dynamic, Engine, EvalAltResult, FnPtr, Map};

// Sandbox limits (docs/parity/script_bindings_plan.md §6).
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

/// Generate an RAII push/pop guard for one of the active-context stacks.
///
/// Why RAII and not manual `push`…`pop()`: Rhai does NOT `catch_unwind` its
/// native calls, so a panic in a script body (or a deep `RefCell` borrow panic,
/// incl. the B1 re-entrancy guard) skips a trailing manual `pop()` — leaving a
/// **stale** entry on the stack. `reset_thread_state` doesn't clear these
/// `latexml_contrib` thread-locals, so the dangling entry survives the cortex
/// per-paper `catch_unwind` and leaks into the next paper. A `Drop` guard pops
/// on EVERY scope exit — normal return, `?` early-return, or unwind. Review M1.
macro_rules! ctx_stack_guard {
  ($guard:ident, $stack:ident, $elem:ty) => {
    #[must_use = "the guard pops on drop; binding it to `_` would pop immediately"]
    struct $guard(());
    impl $guard {
      fn new(entry: $elem) -> Self {
        $stack.with(|c| c.borrow_mut().push(entry));
        Self(())
      }
    }
    impl Drop for $guard {
      fn drop(&mut self) {
        $stack.with(|c| {
          c.borrow_mut().pop();
        });
      }
    }
  };
}

ctx_stack_guard!(CtorCtxGuard, CTOR_CTX, CtorCtx);
ctx_stack_guard!(WhatsitCtxGuard, WHATSIT_CTX, (*mut Whatsit, bool));
ctx_stack_guard!(ScriptCtxGuard, CURRENT_SCRIPT, (Rc<Engine>, Rc<AST>));

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

/// Call a DEFERRED Rhai body (a primitive/macro/constructor/… closure) during
/// digestion, with its `(engine, AST)` pushed onto `CURRENT_SCRIPT` for the
/// duration. This lets the body itself call a registration (`DefPrimitive`,
/// `DefMacro`, …): the nested `wire_now` → `current_script()` resolves this
/// pushed context instead of failing "registration called outside a script
/// load" (#316). Mirrors Perl, where a `def*` sub is callable from anywhere —
/// script load OR digestion. The RAII guard nests correctly for a body that
/// runs another body.
pub(super) fn call_deferred_body<A: rhai::FuncArgs>(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  body: &FnPtr,
  args: A,
) -> std::result::Result<Dynamic, Box<EvalAltResult>> {
  let _script_guard = ScriptCtxGuard::new((engine.clone(), ast.clone()));
  body.call::<Dynamic>(engine, ast, args)
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

/// Run `f` with the active constructor's live document and props — the SINGLE
/// audited site that re-mints the `&mut Document` the core published for the
/// in-flight body (review B1: consolidated here so the `unsafe` has one
/// documented home instead of being scattered across every proxy op).
///
/// RE-ENTRANCY SOUNDNESS (B1 — RESOLVED 2026-06-27, verified, not a band-aid):
/// for a NESTED script construct — `\wrap{\myemph{..}}`, where the outer body's
/// `Document::absorb` re-enters the bridge while its own `&mut self` is still
/// live — this re-mints a *second* `&mut` from a raw pointer while the outer one
/// is parked on the stack. The earlier B1 review feared this was aliasing UB; a
/// careful reborrow analysis (and a Miri model — see below) shows it is **sound**:
/// the nested pointer is a reborrow **descendant** of the outer one, not an alias.
/// The chain is linear: the outer body's `with_doc` reborrows the top pointer →
/// `Document::absorb(&mut self)` reborrows that → the core threads a reborrow of
/// `&mut self` down to the nested constructor (`absorb` → `be_absorbed(self)` →
/// the nested trampoline's `&mut Document`) → that body's `with_doc` reborrows
/// *it*. Because `CTOR_CTX` is a STACK and `current_ctx()` always returns the
/// **innermost** published pointer, every `with_doc` re-mints from a genuine
/// descendant of all parked outer `&mut`s; a descendant reborrow never
/// invalidates its ancestors. The native path is sound for the same reason
/// (reborrow down the call chain); the Rhai closure boundary erases the
/// lifetime, so the round-trip through `*mut` is unavoidable here, but it
/// preserves the reborrow lineage.
///
/// VERIFIED: the exact pattern (thread-local `*mut` stack + RAII guard +
/// `with_doc` re-mint + nested `absorb` reborrowing down) is modeled over a
/// libxml2-free `Doc` in `latexml_core::runtime_bindings_reentrancy_model` and
/// passes Miri under **both Stacked and Tree Borrows, 0 UB** (the real path can't
/// be Miri-checked directly because `Document` is libxml2/FFI). The earlier
/// checked-guard "fix" was correctly REJECTED — there is no UB to guard against,
/// and failing the re-entrant op would deadlock `Document::absorb`'s loop, which
/// requires the nested construction to SUCCEED. Tracked in `docs/SYNC_STATUS.md`
/// "Open tasks #3 / PR #248 B1".
fn with_doc<R>(
  f: impl FnOnce(&mut Document, &SymHashMap<Stored>) -> std::result::Result<R, Box<EvalAltResult>>,
) -> std::result::Result<R, Box<EvalAltResult>> {
  let ctx = current_ctx()?;
  // SAFETY: `ctx.document` is the `&mut Document` the core published for this
  // body (valid for its duration). For the non-nested case this is the unique
  // live `&mut`; for the nested case see the CAVEAT above. `ctx.props` is read
  // only — never mutated through this `*const`.
  let doc = unsafe { &mut *ctx.document };
  let props = unsafe { &*ctx.props };
  f(doc, props)
}

/// Rhai proxy for the live document, passed to a constructor body as its first
/// argument — so a binding reads like the Perl original (`$document->method`).
/// It carries no pointer itself; its methods resolve the active-context, so it
/// is only valid inside a constructor body (a method call outside one is a clean
/// error, never UB).
#[derive(Clone)]
struct DocProxy;

/// Rhai proxy for a document-tree node: wraps the clonable libxml handle
/// directly — no lifetimes, no active context. Handed to closure-form matcher
/// bodies (`DefMathLigature(|node| …)`, read-only) and rewrite-`replace` bodies
/// (mutable — `setAttribute`/`setContent`/`unlink`).
///
/// ⚠ LIFETIME FOOTGUN (review m5): the wrapped handle aliases a live C `xmlNode`
/// owned by the conversion's document tree, and is valid ONLY for the duration
/// of the body that received it. A script that stows a `NodeProxy` in a variable
/// and dereferences it AFTER `unlink()` has detached the node, or after the
/// conversion ends and the tree is freed (`reset_thread_engine`), is a C-level
/// use-after-free (cf. WISDOM #58). Unlike `DocProxy`/`WhatsitProxy` — which
/// resolve their target through the active-context stack on each call and carry
/// no pointer — `NodeProxy` is the only proxy holding a raw tree handle. Use it
/// within the body; never retain it across calls or past `unlink()`.
#[derive(Clone)]
pub(crate) struct NodeProxy(pub(crate) libxml::tree::Node);

/// A Clone-able builder mirroring `std::process::Command`, exposed to Rhai as
/// `Command` (#318) so a trusted binding can shell out to `latexmk`/`dvisvgm`
/// during digestion — Perl `.ltxml` runs `system()` freely; Rhai is sandboxed,
/// so this is the deliberate escape hatch (SAFETY.md: the runtime-bindings
/// feature IS the trust boundary — untrusted deployments drop it). std's
/// `Command` isn't `Clone`, so hold the fields and construct the real one at
/// `.output()`. The Rhai method names mirror std's exactly, so a user reads the
/// `std::process::Command` docs.
#[derive(Clone, Default)]
pub(super) struct RhaiCommand {
  program: String,
  args:    Vec<String>,
  envs:    Vec<(String, String)>,
  cwd:     Option<String>,
}

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
  // (a stack, so a script loading another script nests correctly). RAII pop on
  // every exit incl. a panic out of `run_ast` (review M1).
  let before = WIRED_COUNT.with(|c| *c.borrow());
  let run = {
    let _script_guard = ScriptCtxGuard::new((cached.engine.clone(), cached.ast.clone()));
    cached
      .engine
      .run_ast(&cached.ast)
      .map_err(|e| Error::from(format!("script-binding run error: {e}")))
  };
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
    match v.clone().try_cast::<Digested>() {
      Some(d) => {
        if k.as_str() == "font"
          && let Ok(Some(f)) = d.get_font()
        {
          props.insert("font", Stored::Font(Rc::new(f.into_owned())));
          continue;
        }
        props.insert(k.as_str(), Stored::Digested(d));
      },
      _ => {
        props.insert(k.as_str(), dynamic_to_string(v).into());
      },
    }
  }
  props
}

/// Marshal a counter/whatsit property map back into a Rhai object map (the
/// inverse of [`rhai_map_to_props`]). The single source shared by
/// `RefStepCounter`/`RefStepID`/`RefCurrentID`, which inlined this 3× before
/// (review m4).
fn props_to_map(props: SymHashMap<Stored>) -> Map {
  let mut m = Map::new();
  for (k, v) in props {
    m.insert(arena::to_string(k).into(), stored_to_dynamic(v));
  }
  m
}

/// Map ANY displayable error (latexml `Error`, libxml `Box<dyn Error>`, …) into
/// a Rhai error — the SINGLE boundary-conversion source. Review m4: the inline
/// `Box::<EvalAltResult>::from(e.to_string())` spelling scattered across the
/// proxy ops is routed through here so error formatting has one definition.
fn rhai_err(e: impl std::fmt::Display) -> Box<EvalAltResult> {
  Box::<EvalAltResult>::from(e.to_string())
}

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

#[cfg(test)]
mod tests;
