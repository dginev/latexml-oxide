//! Runtime script bindings via embedded **Rhai** (see `docs/script_bindings_plan.md`).
//!
//! Feature-gated behind `script-bindings` (OFF by default). The ONLY module in
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
//! * **`DefMacro`** (expandable) — body receives args as strings, returns a
//!   string of TeX, re-tokenized via `mouth::tokenize_internal` so an expansion
//!   to `\textit{x}` faithfully yields a control-sequence token.
//! * **`DefConstructor`** (construction) — two forms. A string XML template, or
//!   an imperative body that reads like the Perl original: it receives a
//!   `Document` **proxy** as its first argument (Perl's `$_[0]`) and each digested
//!   argument as an opaque handle, e.g.
//!   `|document, x| { document.openElement("ltx:emph"); document.absorb(x);
//!   document.closeElement("ltx:emph"); }`. The proxy's methods resolve a
//!   thread-local **active context** (the live `&mut Document` + props, published
//!   for the duration of the call; never borrowed across a re-entrant call).
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

use rhai::{Dynamic, Engine, EvalAltResult, FnPtr, Map, AST};

use latexml_core::binding::def::builder::{ConstructorBuilder, EnvironmentBuilder, OptionValue};
use latexml_core::binding::def::dialect::{def_conditional, def_macro, def_primitive};
use latexml_core::common::dimension::Dimension;
use latexml_core::common::numeric_ops::NumericOps;
use latexml_core::definition::conditional::ConditionalOptions;
use latexml_core::definition::expandable::ExpandableOptions;
use latexml_core::definition::math_primitive::MathPrimitiveOptions;
use latexml_core::definition::ConditionalClosure;
use latexml_core::binding::def::replacement;
use latexml_core::common::arena::{self, SymHashMap};
use latexml_core::common::def_parser::parse_prototype;
use latexml_core::common::error::{Error, Result};
use latexml_core::common::number::Number;
use latexml_core::common::store::Stored;
use latexml_core::definition::argument::ArgWrap;
use latexml_core::definition::primitive::PrimitiveOptions;
use latexml_core::definition::{
  BeforeDigestClosure, ConstructionClosure, DigestionClosure, ExpansionBody, ExpansionClosure,
  FontDirective, PrimitiveBody, PrimitiveClosure, PropertiesClosure, ReplacementClosure, Reversion,
};
use latexml_core::digested::Digested;
use latexml_core::document::Document;
use latexml_core::mouth;
use latexml_core::state::Scope;
use latexml_core::tokens::Tokens;
use latexml_core::common::object::Object;
use latexml_core::token::{Catcode, Token};
use latexml_core::whatsit::Whatsit;
use latexml_core::BoxOps;
// `Error!` expands a `Fatal!`/`fatal!` arm (too-many-errors escalation); the
// whole macro chain must be in scope at the expansion site (non-hygienic).
#[allow(unused_imports)]
use latexml_core::{fatal, Fatal};

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
  ast: Rc<AST>,
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
  static WHATSIT_CTX: RefCell<Vec<*mut Whatsit>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy)]
struct CtorCtx {
  document: *mut Document,
  props: *const SymHashMap<Stored>,
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
  WHATSIT_CTX.with(|c| {
    c.borrow()
      .last()
      .copied()
      .ok_or_else(|| Box::<EvalAltResult>::from("whatsit() called outside a hook body"))
  })
}

/// Mini-DSL to expose a `document.<rhai>(qname)` → `Document::<rust>(qname)`
/// method whose result is discarded — the common side-effect-on-element shape.
/// Adding a new such method is then one line; see the registrations in
/// `make_engine`.
macro_rules! doc_qname_method {
  ($engine:ident, $rhai:literal, $rust:ident) => {
    $engine.register_fn(
      $rhai,
      |_d: &mut DocProxy, qname: &str| -> std::result::Result<(), Box<EvalAltResult>> {
        let doc = unsafe { &mut *current_ctx()?.document };
        doc.$rust(qname).map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
        Ok(())
      },
    );
  };
}

/// Build a sandboxed Rhai engine with the binding API registered.
fn make_engine() -> Engine {
  let mut engine = Engine::new();
  engine.set_max_operations(MAX_OPERATIONS);
  engine.set_max_call_levels(MAX_CALL_LEVELS);
  engine.set_max_string_size(MAX_STRING_SIZE);

  // ── registration API (wired to native defs IMMEDIATELY, in script order) ──
  engine.register_fn(
    "DefMacro",
    |proto: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_macro(e, a, proto, body))
    },
  );
  // Option-bag form (Perl's trailing `key => value`s): scalars onto
  // `ExpandableOptions` via the shared mapper.
  engine.register_fn(
    "DefMacro",
    |proto: &str, body: FnPtr, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_macro_opts(e, a, proto, body, opts))
    },
  );
  engine.register_fn(
    "DefPrimitive",
    |proto: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_primitive(e, a, proto, body, PrimitiveOptions::default()))
    },
  );
  engine.register_fn(
    "DefPrimitive",
    |proto: &str, body: FnPtr, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_primitive(e, a, proto, body, primitive_options_from_map(opts)))
    },
  );
  // Class/package option. Mirrors the `DeclareOption!` macro's lowering
  // (setup_binding_language.rs): push the name onto `@declaredoptions` and
  // define a `\ds@<opt>` primitive carrying the body. Installed immediately,
  // so a following `ProcessOptions()` in the same script sees it.
  engine.register_fn(
    "DeclareOption",
    |opt: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_option(e, a, opt, body))
    },
  );

  // ── free-function helpers (prelude names) over core datatypes ──
  // `Tokens` is registered as an opaque pass-through handle (Tier-2): a script
  // produces it with Tokenize and hands it to Digest without inspecting it.
  engine.register_type_with_name::<Tokens>("Tokens");
  engine.register_fn("Tokenize", |s: &str| -> Tokens { mouth::tokenize(s) });
  engine.register_fn("TokenizeInternal", |s: &str| -> Tokens { mouth::tokenize_internal(s) });
  engine.register_fn("Digest", |t: Tokens| -> std::result::Result<(), Box<EvalAltResult>> {
    latexml_core::stomach::digest(t).map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
    Ok(())
  });

  // ── state API (for primitive side-effects / value-reading macros) ──
  // `assign_value` is group-local (TeX default); `assign_global` persists past
  // the enclosing group. (Untrusted-script key-namespacing is a documented
  // follow-up — see the plan's critical re-eval §5.)
  engine.register_fn("assign_value", |key: &str, val: &str| {
    latexml_core::state::assign_value(key, val.to_string(), Some(Scope::Local));
  });
  engine.register_fn("assign_global", |key: &str, val: &str| {
    latexml_core::state::assign_value(key, val.to_string(), Some(Scope::Global));
  });
  // Curated pool helpers, registered 1:1 under their Perl/Rust names so binding
  // bodies read like the originals (e.g. `beforeDigest: || neutralize_font()`).
  engine.register_fn("neutralize_font", latexml_engine::base_utilities::neutralize_font);
  engine.register_fn("lookup_value", |key: &str| -> String {
    match latexml_core::state::lookup_value(key) {
      Some(Stored::String(s)) => arena::to_string(s),
      Some(other) => other.to_string(),
      None => String::new(),
    }
  });

  // ── the binding-language surface, 1:1 under its macro names ──
  // Each registration lowers to the SAME native function its compile-time
  // macro does (setup_binding_language.rs), so a Rhai binding reads like the
  // `.pool`/`_sty.rs` original. String values in, strings/handles out.
  engine.register_fn("AssignValue", |k: &str, v: &str| {
    latexml_core::state::assign_value(k, v.to_string(), None);
  });
  engine.register_fn("AssignValue", |k: &str, v: &str, scope: &str| {
    latexml_core::state::assign_value(k, v.to_string(), scope_of(scope));
  });
  engine.register_fn("LookupString", |k: &str| -> String { latexml_core::state::lookup_string(k) });
  engine.register_fn("LookupNumber", |k: &str| -> i64 {
    latexml_core::state::lookup_number(k).map(|n| n.0).unwrap_or(0)
  });
  engine.register_fn("LookupBool", |k: &str| -> bool { latexml_core::state::lookup_bool(k) });
  engine.register_fn("Let", |a: &str, b: &str| {
    latexml_core::state::let_i(&latexml_core::T_CS!(a), &latexml_core::T_CS!(b), None);
  });
  engine.register_fn("XEquals", |a: &str, b: &str| -> bool {
    latexml_core::state::x_equals(&latexml_core::T_CS!(a), &latexml_core::T_CS!(b))
  });
  engine.register_fn("IsDefined", |cs: &str| -> bool {
    latexml_core::binding::def::dialect::is_defined_token(&latexml_core::T_CS!(cs))
  });
  // RawTeX: process literal TeX as definitions input (the raw-`\def` escape
  // hatch every nontrivial binding uses). TeX: tokenize-internal + digest.
  engine.register_fn("RawTeX", |text: &str| -> std::result::Result<(), Box<EvalAltResult>> {
    latexml_core::stomach::raw_tex(text).map_err(rhai_err)
  });
  engine.register_fn("TeX", |text: &str| -> std::result::Result<(), Box<EvalAltResult>> {
    latexml_core::stomach::digest(mouth::tokenize_internal(text)).map_err(rhai_err)?;
    Ok(())
  });
  engine.register_fn("Expand", |t: Tokens| -> std::result::Result<Tokens, Box<EvalAltResult>> {
    latexml_core::gullet::do_expand(t).map_err(rhai_err)
  });
  engine.register_fn(
    "ExpandPartially",
    |t: Tokens| -> std::result::Result<Tokens, Box<EvalAltResult>> {
      latexml_core::gullet::do_expand_partially(t).map_err(rhai_err)
    },
  );
  engine.register_fn("UnTeX", |t: Tokens| -> String { t.untex() });
  // DigestText: digest a TeX string into a `Digested` handle — the workhorse
  // of `properties` closures that precompute content (e.g. IEEEproof's title).
  engine.register_fn(
    "DigestText",
    |s: &str| -> std::result::Result<Digested, Box<EvalAltResult>> {
      latexml_core::binding::content::digest_text(mouth::tokenize_internal(s)).map_err(rhai_err)
    },
  );
  // ToString/ToAttribute/Revert on a Digested handle (Perl ToString/Revert).
  engine.register_fn("ToString", |d: Digested| -> String { d.to_string() });
  engine.register_fn("ToAttribute", |d: Digested| -> String { d.to_attribute() });
  engine.register_fn("Revert", |d: Digested| -> std::result::Result<Tokens, Box<EvalAltResult>> {
    d.revert().map_err(rhai_err)
  });
  engine.register_fn("Today", || -> std::result::Result<String, Box<EvalAltResult>> {
    latexml_engine::base_utilities::today().map_err(rhai_err)
  });
  engine.register_fn("Warn", |cat: &str, obj: &str, msg: &str| {
    latexml_core::Warn!(cat, obj, msg.to_string());
  });
  engine.register_fn(
    "Error",
    |cat: &str, obj: &str, msg: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      // Error! escalates to Fatal past MAX_ERRORS (a latexml `Err`) — surface
      // that to the script as a Rhai error so conversion aborts cleanly.
      let res: Result<()> = (|| {
        latexml_core::Error!(cat, obj, msg.to_string());
        Ok(())
      })();
      res.map_err(rhai_err)
    },
  );

  // ── counters (counter_dialect, the NewCounter!/StepCounter!/… family) ──
  engine.register_fn("NewCounter", |c: &str| -> std::result::Result<(), Box<EvalAltResult>> {
    latexml_core::binding::counter::dialect::new_counter(c, "", None).map_err(rhai_err)
  });
  engine.register_fn(
    "NewCounter",
    |c: &str, within: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::new_counter(c, within, None).map_err(rhai_err)
    },
  );
  engine.register_fn("StepCounter", |c: &str| -> std::result::Result<(), Box<EvalAltResult>> {
    latexml_core::binding::counter::dialect::step_counter(c, false).map_err(rhai_err)
  });
  engine.register_fn("ResetCounter", |c: &str| -> std::result::Result<(), Box<EvalAltResult>> {
    latexml_core::binding::counter::dialect::reset_counter(&latexml_core::T_LETTER!(c))
      .map_err(rhai_err)
  });
  engine.register_fn(
    "AddToCounter",
    |c: &str, n: i64| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::add_to_counter(c, Number(n)).map_err(rhai_err)
    },
  );
  engine.register_fn("CounterValue", |c: &str| -> std::result::Result<i64, Box<EvalAltResult>> {
    latexml_core::binding::counter::dialect::counter_value(c).map(|n| n.0).map_err(rhai_err)
  });
  // RefStepCounter: returns the refnum/id property map (Digested values come
  // back as handles a `properties` closure can return directly — the amsmath
  // `properties => ref_step_counter("equation")` idiom).
  engine.register_fn(
    "RefStepCounter",
    |c: &str| -> std::result::Result<Map, Box<EvalAltResult>> {
      let props = latexml_core::binding::counter::dialect::ref_step_counter(c, false).map_err(rhai_err)?;
      let mut m = Map::new();
      for (k, v) in props {
        m.insert(arena::to_string(k).into(), stored_to_dynamic(v));
      }
      Ok(m)
    },
  );

  // ── further definition forms (DefRegister/DefKeyVal/DefMath/DefLigature) ──
  // These run when the script executes (per conversion, like a Perl .ltxml),
  // lowering to the same dialect functions their compile-time macros use.
  engine.register_fn(
    "DefRegister",
    |proto: &str, v: i64| -> std::result::Result<(), Box<EvalAltResult>> {
      let (cs, params) = parse_prototype(proto, true).map_err(rhai_err)?;
      latexml_core::binding::def::dialect::def_register(cs, params, Number(v), None)
        .map_err(rhai_err)
    },
  );
  // String value = a dimension spec ("5pt", "0.4em", …) → Dimension register.
  engine.register_fn(
    "DefRegister",
    |proto: &str, spec: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let (cs, params) = parse_prototype(proto, true).map_err(rhai_err)?;
      let dim = Dimension::new_f64(Dimension::spec_to_f64(spec).map_err(rhai_err)?);
      latexml_core::binding::def::dialect::def_register(cs, params, dim, None).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DefKeyVal",
    |keyset: &str, key: &str, vtype: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::keyval::define(latexml_core::keyval::KeyvalConfig {
        prefix: "KV",
        keyset,
        key,
        vtype,
        default: None,
        ..latexml_core::keyval::KeyvalConfig::default()
      })
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DefKeyVal",
    |keyset: &str,
     key: &str,
     vtype: &str,
     default: &str|
     -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::keyval::define(latexml_core::keyval::KeyvalConfig {
        prefix: "KV",
        keyset,
        key,
        vtype,
        default: Some(default),
        ..latexml_core::keyval::KeyvalConfig::default()
      })
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DefLigature",
    |pattern: &str, replacement: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let regex_compiled = regex::Regex::new(pattern)
        .map_err(|e| Box::<EvalAltResult>::from(format!("DefLigature bad regex: {e}")))?;
      let replacement = replacement.to_string();
      latexml_core::state::unshift_value("TEXT_LIGATURES", vec![latexml_core::ligature::Ligature {
        id:        latexml_core::state::generate_ligature_id(),
        regex:     Some(pattern.to_string()),
        code:      Some(Rc::new(move |text| {
          regex_compiled.replace_all(text, replacement.as_str()).to_string()
        })),
        font_test: None,
        matcher:   None,
      }]);
      Ok(())
    },
  );
  // DefMath: presentation-string form (the dominant one) + option bag with the
  // macro's scalar option set.
  engine.register_fn(
    "DefMath",
    |proto: &str, presentation: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let (cs, params) = parse_prototype(proto, true).map_err(rhai_err)?;
      latexml_core::binding::def::dialect::def_math(
        cs,
        params,
        presentation.to_string(),
        MathPrimitiveOptions::default(),
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DefMath",
    |proto: &str, presentation: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      let (cs, params) = parse_prototype(proto, true).map_err(rhai_err)?;
      let options = math_options_from_map(opts).map_err(rhai_err)?;
      latexml_core::binding::def::dialect::def_math(cs, params, presentation.to_string(), options)
        .map_err(rhai_err)
    },
  );
  // DefConditional: the test is a Rhai closure receiving args as strings and
  // returning a bool.
  engine.register_fn(
    "DefConditional",
    |proto: &str, test: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_conditional(e, a, proto, test))
    },
  );

  // ── package/class machinery (content.rs, the RequirePackage!/… family) ──
  engine.register_fn(
    "RequirePackage",
    |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::require_package(
        name,
        latexml_core::binding::content::RequireOptions::default(),
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "RequirePackage",
    |name: &str, options: rhai::Array| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::require_package(
        name,
        latexml_core::binding::content::RequireOptions {
          options: options.into_iter().map(dynamic_to_string).collect(),
          ..latexml_core::binding::content::RequireOptions::default()
        },
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn("LoadClass", |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
    latexml_core::binding::content::load_class(name, Vec::new(), Tokens::default())
      .map_err(rhai_err)
  });
  engine.register_fn(
    "LoadClass",
    |name: &str, options: rhai::Array| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::load_class(
        name,
        options.into_iter().map(dynamic_to_string).collect(),
        Tokens::default(),
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn("ProcessOptions", || -> std::result::Result<(), Box<EvalAltResult>> {
    latexml_core::binding::content::process_options(false, &[]).map_err(rhai_err)
  });
  // ProcessOptions(true) = the `\ProcessOptions*` in-order variant.
  engine.register_fn(
    "ProcessOptions",
    |inorder: bool| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::process_options(inorder, &[]).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "ExecuteOptions",
    |options: rhai::Array| -> std::result::Result<(), Box<EvalAltResult>> {
      let opts: Vec<String> = options.into_iter().map(dynamic_to_string).collect();
      let refs: Vec<&str> = opts.iter().map(String::as_str).collect();
      latexml_core::binding::content::execute_options(&refs).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "PassOptions",
    |name: &str, ext: &str, options: rhai::Array| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::pass_options(
        name,
        ext,
        options.into_iter().map(dynamic_to_string).collect(),
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn("RequireResource", |resource: &str| {
    latexml_core::binding::content::require_resource(latexml_core::document::resource::Resource {
      name: resource.to_string(),
      ..latexml_core::document::resource::Resource::default()
    });
  });
  // Tag: the scalar subset (autoOpen/autoClose) of TagOptions.
  engine.register_fn("Tag", |tag: &str, opts: Map| {
    let mut options = latexml_core::document::tag::TagOptions::default();
    for (key, val) in opts {
      match key.as_str() {
        "autoOpen" => options.auto_open = val.as_bool().ok(),
        "autoClose" => options.auto_close = val.as_bool().ok(),
        _ => {},
      }
    }
    latexml_core::binding::content::install_tag(tag, options);
  });
  // MergeFont: merge the given partial font into the current one (Perl
  // `MergeFont(family=>…)`); string keys family/series/shape/size.
  engine.register_fn("MergeFont", |opts: Map| {
    latexml_core::binding::content::merge_font(font_from_rhai_map(opts));
  });
  engine.register_fn(
    "DefConstructor",
    |proto: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_constructor(e, a, proto, body))
    },
  );
  // String-template form (the dominant constructor dialect): the second arg is
  // an XML template instead of a closure. Rhai dispatches by the arg's type.
  engine.register_fn(
    "DefConstructor",
    |proto: &str, template: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|_e, _a| wire_constructor_template(proto, template.to_string()))
    },
  );
  // Option-bag forms: a trailing Rhai object map `#{ mode: …, afterDigest: |…| … }`
  // — the analog of Perl's `%options` / the `DefConstructor!` macro's `key => value`
  // (named, any order, omittable; values may be strings or closures).
  engine.register_fn(
    "DefConstructor",
    |proto: &str, template: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| {
        wire_constructor_opts(e, a, proto, ConstructorRepl::Template(template.to_string()), opts)
      })
    },
  );
  engine.register_fn(
    "DefConstructor",
    |proto: &str, body: FnPtr, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_constructor_opts(e, a, proto, ConstructorRepl::Closure(body), opts))
    },
  );

  // ── DefEnvironment: same four shapes as DefConstructor; the prototype is the
  // `DefEnvironment!` form (`"{name}"` / `"{name}{}…"`), the template will
  // typically reference `#body`. ──
  engine.register_fn(
    "DefEnvironment",
    |proto: &str, template: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| {
        wire_environment(
          e,
          a,
          proto,
          ConstructorRepl::Template(template.to_string()),
          Map::new(),
        )
      })
    },
  );
  engine.register_fn(
    "DefEnvironment",
    |proto: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_environment(e, a, proto, ConstructorRepl::Closure(body), Map::new()))
    },
  );
  engine.register_fn(
    "DefEnvironment",
    |proto: &str, template: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| {
        wire_environment(e, a, proto, ConstructorRepl::Template(template.to_string()), opts)
      })
    },
  );
  engine.register_fn(
    "DefEnvironment",
    |proto: &str, body: FnPtr, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_environment(e, a, proto, ConstructorRepl::Closure(body), opts))
    },
  );

  // ── document proxy: methods mirror Perl's `$document->method` idiom ──
  // The body receives `document` (a DocProxy) as its first arg, and each digested
  // argument as an opaque `Digested` handle it can pass back to `document.absorb`.
  engine.register_type_with_name::<DocProxy>("Document");
  engine.register_type_with_name::<Digested>("Digested");

  doc_qname_method!(engine, "closeElement", close_element);
  doc_qname_method!(engine, "maybeCloseElement", maybe_close_element);

  engine.register_fn(
    "openElement",
    |_d: &mut DocProxy, tag: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let doc = unsafe { &mut *current_ctx()?.document };
      doc.open_element(tag, None, None).map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
      Ok(())
    },
  );
  engine.register_fn(
    "setAttribute",
    |_d: &mut DocProxy, key: &str, val: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let doc = unsafe { &mut *current_ctx()?.document };
      let mut node = doc.get_node().clone();
      doc
        .set_attribute(&mut node, key, val)
        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
      Ok(())
    },
  );
  engine.register_fn(
    "absorbString",
    |_d: &mut DocProxy, s: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let ctx = current_ctx()?;
      let doc = unsafe { &mut *ctx.document };
      let props = unsafe { &*ctx.props };
      doc.absorb_string(s, props).map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
      Ok(())
    },
  );
  engine.register_fn(
    "absorb",
    |_d: &mut DocProxy, arg: Digested| -> std::result::Result<(), Box<EvalAltResult>> {
      let doc = unsafe { &mut *current_ctx()?.document };
      doc.absorb(&arg, None).map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
      Ok(())
    },
  );
  // Absorb a whatsit property at the current point — the imperative analog of a
  // template's `#name` hole at content position. The workhorse is
  // `document.absorbProperty("body")` inside an imperative `DefEnvironment`
  // (mirroring natives like `{center}`'s `sub[document, _args, props]` body).
  engine.register_fn(
    "absorbProperty",
    |_d: &mut DocProxy, name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let ctx = current_ctx()?;
      let doc = unsafe { &mut *ctx.document };
      let props = unsafe { &*ctx.props };
      if let Some(stored) = props.get(name) {
        let dig: Option<Digested> = stored.into();
        if let Some(ref d) = dig {
          doc.absorb(d, None).map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
        }
      }
      Ok(())
    },
  );

  // ── whatsit proxy: reached from a hook body via `whatsit()` ──
  engine.register_type_with_name::<WhatsitProxy>("Whatsit");
  engine.register_fn("whatsit", || WhatsitProxy);
  // The n-th (1-based) argument as its TeX-source string.
  engine.register_fn(
    "argString",
    |_w: &mut WhatsitProxy, n: i64| -> std::result::Result<String, Box<EvalAltResult>> {
      let w = unsafe { &*current_whatsit()? };
      match w.get_arg(n as usize) {
        Some(d) => d.untex().map_err(|e| Box::<EvalAltResult>::from(e.to_string())),
        None => Ok(String::new()),
      }
    },
  );
  // Set a whatsit property from a hook body (Perl `$whatsit->setProperty(k, v)`,
  // e.g. plain `\footnote`'s afterDigest routing its mark arg to `mark`/`prenote`).
  // The value lands as a string `Stored`, which the template interpreter renders
  // at attribute (`to_attribute`), content (absorb), and truth-test positions.
  engine.register_fn(
    "setProperty",
    |_w: &mut WhatsitProxy, key: &str, val: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let w = unsafe { &mut *current_whatsit()? };
      w.set_property(key, val.to_string());
      Ok(())
    },
  );
  // Read a whatsit property as a string ("" when absent) — Perl `getProperty`.
  engine.register_fn(
    "propertyString",
    |_w: &mut WhatsitProxy, key: &str| -> std::result::Result<String, Box<EvalAltResult>> {
      let w = unsafe { &*current_whatsit()? };
      Ok(w.get_property_string(key))
    },
  );

  engine
}

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
      let cs = CachedScript { engine: Rc::new(engine), ast: Rc::new(ast) };
      SCRIPT_CACHE.with(|c| {
        c.borrow_mut().insert(src.to_string(), cs.clone());
      });
      cs
    },
  };

  // Publish the script handles for the registration closures, run, unpublish
  // (a stack, so a script loading another script nests correctly).
  CURRENT_SCRIPT.with(|c| c.borrow_mut().push((cached.engine.clone(), cached.ast.clone())));
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

/// Install one `DefMacro` registration as a native expandable definition.
fn wire_macro(engine: &Rc<Engine>, ast: &Rc<AST>, proto: &str, body: FnPtr) -> Result<()> {
  let (cs, paramlist) = parse_prototype(proto, true)?;
  let cs_name = cs.to_string();
  let engine = engine.clone();
  let ast = ast.clone();

  let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| -> Result<Tokens> {
    let dyn_args: Vec<Dynamic> = args.into_iter().map(arg_to_dynamic).collect();
    let ret: Dynamic = body
      .call::<Dynamic>(&engine, &ast, dyn_args)
      .map_err(|e| Error::from(format!("script macro {cs_name}: {e}")))?;
    Ok(mouth::tokenize_internal(&dynamic_to_string(ret)))
  });

  def_macro(cs, paramlist, ExpansionBody::Closure(closure), None)?;
  Ok(())
}

/// The imperative-body replacement closure: publishes the active context, calls
/// the body as `|document, arg1, …|`, pops the context. Shared by every
/// constructor form that uses a closure body.
fn closure_replacement(
  body: FnPtr,
  engine: Rc<Engine>,
  ast: Rc<AST>,
  cs_name: String,
) -> ReplacementClosure {
  Rc::new(move |document: &mut Document, args: &Vec<Option<Digested>>, props| -> Result<()> {
    // Each `with` is a short borrow; the body's document ops re-borrow freshly.
    CTOR_CTX.with(|c| {
      c.borrow_mut().push(CtorCtx { document, props });
    });
    // `document` first (Perl's `$_[0]`), then each digested arg as a handle.
    let mut call_args: Vec<Dynamic> = Vec::with_capacity(args.len() + 1);
    call_args.push(Dynamic::from(DocProxy));
    for a in args {
      call_args.push(match a {
        Some(d) => Dynamic::from(d.clone()),
        None => Dynamic::UNIT,
      });
    }
    let result = body.call::<Dynamic>(&engine, &ast, call_args);
    CTOR_CTX.with(|c| {
      c.borrow_mut().pop();
    });
    result
      .map(|_| ())
      .map_err(|e| Error::from(format!("script constructor {cs_name}: {e}")))
  })
}

/// The string-template replacement closure. The template is parsed **once** here
/// (at wire time) into the shared `ReplacementOp` AST — the same parser the
/// compile-time codegen uses (#171) — and the cached AST is interpreted per
/// invocation. This eliminates the former per-invocation byte-scan and removes
/// the second, divergent template implementation.
fn template_replacement(template: &str) -> Result<ReplacementClosure> {
  let ops = Rc::new(replacement::parse_replacement(template)?);
  Ok(Rc::new(move |document: &mut Document, args, props| {
    replacement::apply_ops(&ops, document, args, props)
  }))
}

/// Install one `DefConstructor` (imperative body, no options) via the shared
/// `ConstructorBuilder`.
fn wire_constructor(engine: &Rc<Engine>, ast: &Rc<AST>, proto: &str, body: FnPtr) -> Result<()> {
  let repl = closure_replacement(body, engine.clone(), ast.clone(), proto.to_string());
  ConstructorBuilder::new(proto)?.replacement(repl).install()
}

/// Install a `DefConstructor` with an option bag (`#{ mode, afterDigest, … }`)
/// through the shared `ConstructorBuilder` — the *same* builder the
/// `DefConstructor!` macro targets, so the two front-ends cannot drift. Scalar
/// options route through the builder's single-source `set_option`; closure
/// options through its typed setters (here, `after_digest`).
fn wire_constructor_opts(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  repl: ConstructorRepl,
  opts: Map,
) -> Result<()> {
  let replacement = match repl {
    ConstructorRepl::Template(t) => template_replacement(&t)?,
    ConstructorRepl::Closure(b) => closure_replacement(b, engine.clone(), ast.clone(), proto.to_string()),
  };
  let builder = ConstructorBuilder::new(proto)?.replacement(replacement);
  apply_opts(builder, opts, engine, ast)?.install()
}

/// Install one `DefEnvironment` registration (template or imperative body, with
/// an option bag — possibly empty) through the shared `EnvironmentBuilder`.
/// The imperative body reaches `#body` via `document.absorbProperty("body")`.
fn wire_environment(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  repl: ConstructorRepl,
  opts: Map,
) -> Result<()> {
  let replacement = match repl {
    ConstructorRepl::Template(t) => template_replacement(&t)?,
    ConstructorRepl::Closure(b) => closure_replacement(b, engine.clone(), ast.clone(), proto.to_string()),
  };
  let builder = EnvironmentBuilder::new(proto)?.replacement(replacement);
  apply_opts(builder, opts, engine, ast)?.install()
}

/// The builder surface the option-bag loop needs — implemented for both core
/// builders so [`apply_opts`] is written once. (A local trait over the foreign
/// builder types; new closure options are added in `apply_opts` + one impl line
/// per builder.)
trait BindingBuilder: Sized {
  fn set_option(self, key: &str, value: OptionValue) -> Result<Self>;
  fn after_digest(self, hook: DigestionClosure) -> Self;
  fn after_digest_begin(self, hook: DigestionClosure) -> Self;
  fn before_digest(self, hook: BeforeDigestClosure) -> Self;
  fn before_digest_end(self, hook: BeforeDigestClosure) -> Self;
  fn before_construct(self, hook: ConstructionClosure) -> Self;
  fn after_construct(self, hook: ConstructionClosure) -> Self;
  fn properties(self, props: PropertiesClosure) -> Self;
  fn reversion(self, rev: Reversion) -> Self;
  fn font(self, font: FontDirective) -> Self;
  // (`install` stays inherent on each builder: call sites get the concrete
  // type back from `apply_opts`, so a trait method would be dead code.)
}

macro_rules! impl_binding_builder {
  ($t:ty) => {
    impl BindingBuilder for $t {
      fn set_option(self, key: &str, value: OptionValue) -> Result<Self> {
        <$t>::set_option(self, key, value)
      }
      fn after_digest(self, hook: DigestionClosure) -> Self { <$t>::after_digest(self, hook) }
      fn after_digest_begin(self, hook: DigestionClosure) -> Self {
        <$t>::after_digest_begin(self, hook)
      }
      fn before_digest(self, hook: BeforeDigestClosure) -> Self {
        <$t>::before_digest(self, hook)
      }
      fn before_digest_end(self, hook: BeforeDigestClosure) -> Self {
        <$t>::before_digest_end(self, hook)
      }
      fn before_construct(self, hook: ConstructionClosure) -> Self {
        <$t>::before_construct(self, hook)
      }
      fn after_construct(self, hook: ConstructionClosure) -> Self {
        <$t>::after_construct(self, hook)
      }
      fn properties(self, props: PropertiesClosure) -> Self { <$t>::properties(self, props) }
      fn reversion(self, rev: Reversion) -> Self { <$t>::reversion(self, rev) }
      fn font(self, font: FontDirective) -> Self { <$t>::font(self, font) }
    }
  };
}
impl_binding_builder!(ConstructorBuilder);
impl_binding_builder!(EnvironmentBuilder);

/// Apply a Rhai option bag onto a builder: closure options become trampolines
/// (typed setters), `properties` also accepts a static map, scalars route
/// through the builder's single-source `set_option`. Shared by the constructor
/// and environment front-ends.
fn apply_opts<B: BindingBuilder>(
  mut builder: B,
  opts: Map,
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
) -> Result<B> {
  for (key, val) in opts {
    if let Some(fp) = val.clone().try_cast::<FnPtr>() {
      // Closure option → typed builder setter (front-end builds the closure).
      match key.as_str() {
        "afterDigest" => {
          builder = builder.after_digest(after_digest_trampoline(fp, engine.clone(), ast.clone()));
        },
        "afterDigestBegin" => {
          builder =
            builder.after_digest_begin(after_digest_trampoline(fp, engine.clone(), ast.clone()));
        },
        "properties" => {
          builder = builder.properties(properties_trampoline(fp, engine.clone(), ast.clone()));
        },
        "beforeDigest" => {
          builder = builder.before_digest(before_digest_trampoline(fp, engine.clone(), ast.clone()));
        },
        "beforeDigestEnd" => {
          builder =
            builder.before_digest_end(before_digest_trampoline(fp, engine.clone(), ast.clone()));
        },
        "beforeConstruct" => {
          builder = builder.before_construct(construction_trampoline(fp, engine.clone(), ast.clone()));
        },
        "afterConstruct" => {
          builder = builder.after_construct(construction_trampoline(fp, engine.clone(), ast.clone()));
        },
        // Unknown closure options are silently ignored (forgiving, like Perl
        // %options; the builder's set_option does the same for scalars).
        _ => {},
      }
    } else if key.as_str() == "properties" && val.is_map() {
      // Static property map (Perl's `properties => { key => value, … }`).
      let map = val.cast::<Map>();
      builder = builder.properties(Rc::new(move |_args| Ok(rhai_map_to_props(map.clone()))));
    } else if key.as_str() == "reversion" {
      // String reversion (`reversion => "\\begin{x}#1\\end{x}"`, "" disables).
      builder =
        builder.reversion(Reversion::Tokens(mouth::tokenize_internal(&dynamic_to_string(val))));
    } else if key.as_str() == "font" && val.is_map() {
      // Partial-font directive (`font => { family => 'typewriter', … }`).
      let font = font_from_rhai_map(val.cast::<Map>());
      builder = builder.font(FontDirective::Asset(Rc::new(font)));
    } else if let Some(ov) = dynamic_to_option_value(&val) {
      // Scalar option → the builder's generic, single-source `set_option`.
      builder = builder.set_option(key.as_str(), ov)?;
    }
  }
  Ok(builder)
}

/// Build a `before/afterConstruct` trampoline: the body runs with the live
/// document published as active context (so `document.*` proxy calls work);
/// the whatsit itself is not yet exposed there (read-only marshaling TBD).
fn construction_trampoline(fp: FnPtr, engine: Rc<Engine>, ast: Rc<AST>) -> ConstructionClosure {
  Rc::new(move |document: &mut Document, whatsit: &Whatsit| -> Result<()> {
    CTOR_CTX.with(|c| {
      c.borrow_mut().push(CtorCtx { document, props: whatsit.get_properties() });
    });
    let result = fp.call::<Dynamic>(&engine, &ast, (Dynamic::from(DocProxy),));
    CTOR_CTX.with(|c| {
      c.borrow_mut().pop();
    });
    let _: Dynamic = result.map_err(|e| Error::from(format!("script afterConstruct: {e}")))?;
    Ok(())
  })
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
      "size" => font.size = val.as_float().ok().or_else(|| val.as_int().ok().map(|i| i as f64)),
      _ => {},
    }
  }
  font
}

/// Build an `afterDigest` trampoline: publish the whatsit so a parameterless body
/// can reach it via `whatsit()`, call the Rhai closure, pop.
fn after_digest_trampoline(fp: FnPtr, engine: Rc<Engine>, ast: Rc<AST>) -> DigestionClosure {
  Rc::new(move |w: &mut Whatsit| -> Result<Vec<Digested>> {
    WHATSIT_CTX.with(|c| c.borrow_mut().push(w as *mut Whatsit));
    let r = fp.call::<Dynamic>(&engine, &ast, ());
    WHATSIT_CTX.with(|c| {
      c.borrow_mut().pop();
    });
    let _: Dynamic = r.map_err(|e| Error::from(format!("script afterDigest: {e}")))?;
    Ok(Vec::new())
  })
}

/// Build a `beforeDigest` trampoline (Perl `beforeDigest => sub {…}`): runs the
/// parameterless Rhai closure before the constructor's arguments are digested
/// (state/font side-effects, e.g. `neutralize_font()`); contributes no boxes.
fn before_digest_trampoline(fp: FnPtr, engine: Rc<Engine>, ast: Rc<AST>) -> BeforeDigestClosure {
  Rc::new(move || -> Result<Vec<Digested>> {
    let _: Dynamic = fp
      .call::<Dynamic>(&engine, &ast, ())
      .map_err(|e| Error::from(format!("script beforeDigest: {e}")))?;
    Ok(Vec::new())
  })
}

/// Build a `properties` trampoline (Perl `properties => sub {…}`): the Rhai
/// closure receives each digested argument as its TeX-source string (`()` for an
/// omitted optional) and returns a map; its entries become the whatsit's
/// properties as string `Stored`s, ready for the template's `#name` holes.
fn properties_trampoline(fp: FnPtr, engine: Rc<Engine>, ast: Rc<AST>) -> PropertiesClosure {
  Rc::new(move |args: &Vec<Option<Digested>>| -> Result<SymHashMap<Stored>> {
    let dyn_args: Vec<Dynamic> = args
      .iter()
      .map(|a| match a {
        Some(d) => Dynamic::from(d.untex().unwrap_or_default()),
        None => Dynamic::UNIT,
      })
      .collect();
    let ret: Dynamic = fp
      .call::<Dynamic>(&engine, &ast, dyn_args)
      .map_err(|e| Error::from(format!("script properties: {e}")))?;
    if ret.is_unit() {
      Ok(SymHashMap::default())
    } else if ret.is_map() {
      Ok(rhai_map_to_props(ret.cast::<Map>()))
    } else {
      Err(Error::from("script properties: body must return a map (or unit)"))
    }
  })
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

/// Map a Rhai scalar `Dynamic` to a builder `OptionValue` (string/bool/int).
fn dynamic_to_option_value(v: &Dynamic) -> Option<OptionValue> {
  if v.is_string() {
    Some(OptionValue::Str(v.clone().into_string().unwrap_or_default()))
  } else if v.is_bool() {
    v.as_bool().ok().map(OptionValue::Bool)
  } else {
    v.as_int().ok().map(OptionValue::Int)
  }
}

/// Install one `DefPrimitive` registration as a native primitive whose body
/// runs at digestion time for side-effects (state assignments, etc.). The body
/// receives args as strings and its return value is ignored (pure side-effect);
/// the primitive contributes no boxes.
fn wire_primitive(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  body: FnPtr,
  options: PrimitiveOptions,
) -> Result<()> {
  let (cs, paramlist) = parse_prototype(proto, true)?;
  let cs_name = cs.to_string();
  let engine = engine.clone();
  let ast = ast.clone();

  let closure: PrimitiveClosure = Rc::new(move |args: Vec<ArgWrap>| -> Result<Vec<Digested>> {
    let dyn_args: Vec<Dynamic> = args.into_iter().map(arg_to_dynamic).collect();
    let _: Dynamic = body
      .call::<Dynamic>(&engine, &ast, dyn_args)
      .map_err(|e| Error::from(format!("script primitive {cs_name}: {e}")))?;
    Ok(Vec::new())
  });

  def_primitive(cs, paramlist, Some(PrimitiveBody::Closure(closure)), options)?;
  Ok(())
}

/// `DefMacro` with an option bag: scalars onto `ExpandableOptions`.
fn wire_macro_opts(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  body: FnPtr,
  opts: Map,
) -> Result<()> {
  let (cs, paramlist) = parse_prototype(proto, true)?;
  let cs_name = cs.to_string();
  let engine = engine.clone();
  let ast = ast.clone();
  let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| -> Result<Tokens> {
    let dyn_args: Vec<Dynamic> = args.into_iter().map(arg_to_dynamic).collect();
    let ret: Dynamic = body
      .call::<Dynamic>(&engine, &ast, dyn_args)
      .map_err(|e| Error::from(format!("script macro {cs_name}: {e}")))?;
    Ok(mouth::tokenize_internal(&dynamic_to_string(ret)))
  });
  def_macro(cs, paramlist, ExpansionBody::Closure(closure), Some(expandable_options_from_map(opts)))?;
  Ok(())
}

/// Map a Rhai option bag onto `ExpandableOptions` (the `DefMacro!` scalar set).
fn expandable_options_from_map(opts: Map) -> ExpandableOptions {
  let mut o = ExpandableOptions::default();
  for (key, val) in opts {
    let b = val.as_bool().unwrap_or(false);
    match key.as_str() {
      "locked" => o.locked = b,
      "protected" => o.protected = b,
      "outer" => o.outer = b,
      "long" => o.long = b,
      "mathactive" => o.mathactive = b,
      "robust" => o.robust = b,
      "scope" => o.scope = scope_of(&dynamic_to_string(val.clone())),
      "alias" => o.alias = Some(dynamic_to_string(val.clone())),
      _ => {},
    }
  }
  o
}

/// Map a Rhai option bag onto `PrimitiveOptions` (the `DefPrimitive!` scalar set).
fn primitive_options_from_map(opts: Map) -> PrimitiveOptions {
  let mut o = PrimitiveOptions::default();
  for (key, val) in opts {
    let b = val.as_bool().unwrap_or(false);
    match key.as_str() {
      "bounded" => o.bounded = b,
      "isPrefix" => o.is_prefix = b,
      "requireMath" => o.require_math = b,
      "forbidMath" => o.forbid_math = b,
      "robust" => o.robust = b,
      "locked" => o.locked = b,
      "enterHorizontal" => o.enter_horizontal = b,
      "leaveHorizontal" => o.leave_horizontal = b,
      "scope" => o.scope = scope_of(&dynamic_to_string(val.clone())),
      "mode" => o.mode = Some(dynamic_to_string(val.clone())),
      "nargs" => o.nargs = val.as_int().ok().map(|i| i as usize),
      _ => {},
    }
  }
  o
}

/// Install one `DefConditional` registration: the Rhai test closure receives
/// each argument as its TeX-source string and must return a bool (mirrors the
/// `DefConditional!` macro's `sub` test → `def_conditional`).
fn wire_conditional(engine: &Rc<Engine>, ast: &Rc<AST>, proto: &str, test: FnPtr) -> Result<()> {
  let (cs, paramlist) = parse_prototype(proto, true)?;
  let cs_name = cs.to_string();
  let engine = engine.clone();
  let ast = ast.clone();
  let closure: ConditionalClosure = Rc::new(move |args: Vec<ArgWrap>| -> Result<bool> {
    let dyn_args: Vec<Dynamic> = args.into_iter().map(arg_to_dynamic).collect();
    let ret: Dynamic = test
      .call::<Dynamic>(&engine, &ast, dyn_args)
      .map_err(|e| Error::from(format!("script conditional {cs_name}: {e}")))?;
    Ok(ret.as_bool().unwrap_or(false))
  });
  def_conditional(cs, paramlist, Some(closure), ConditionalOptions::default())
}

/// Map a Rhai option bag onto `MathPrimitiveOptions` (the `DefMath!` scalar
/// option set; unknown keys are ignored, matching Perl %options forgiveness).
fn math_options_from_map(opts: Map) -> Result<MathPrimitiveOptions> {
  let mut o = MathPrimitiveOptions::default();
  for (key, val) in opts {
    let s = || dynamic_to_string(val.clone());
    let b = || val.as_bool().unwrap_or(false);
    match key.as_str() {
      "name" => o.name = Some(s()),
      "meaning" => o.meaning = Some(s()),
      "omcd" => o.omcd = Some(s()),
      "role" => o.role = Some(s()),
      "operator_role" => o.operator_role = Some(s()),
      "mathstyle" => o.mathstyle = Some(s()),
      "scriptpos" => o.scriptpos = Some(s()),
      "mode" => o.mode = Some(s()),
      "alias" => o.alias = Some(s()),
      "revert_as" => o.revert_as = Some(std::borrow::Cow::Owned(s())),
      "bounded" => o.bounded = b(),
      "requireMath" => o.require_math = b(),
      "forbidMath" => o.forbid_math = b(),
      "isPrefix" => o.is_prefix = b(),
      "reorder" => o.reorder = b(),
      "dual" => o.dual = b(),
      "nogroup" => o.nogroup = b(),
      "stretchy" => o.stretchy = val.as_bool().ok(),
      "operator_stretchy" => o.operator_stretchy = val.as_bool().ok(),
      "protected" => o.protected = b(),
      "robust" => o.robust = b(),
      "locked" => o.locked = b(),
      "hide_content_reversion" => o.hide_content_reversion = b(),
      "lpadding" => o.lpadding = val.as_int().ok().map(|i| i as usize),
      "rpadding" => o.rpadding = val.as_int().ok().map(|i| i as usize),
      _ => {},
    }
  }
  Ok(o)
}

/// Install a `DeclareOption` registration. Mirrors the `DeclareOption!` macro
/// (`setup_binding_language.rs`): record the option name in `@declaredoptions`
/// and define a `\ds@<opt>` primitive whose body runs when the option fires.
fn wire_option(engine: &Rc<Engine>, ast: &Rc<AST>, opt: &str, body: FnPtr) -> Result<()> {
  latexml_core::state::push_value("@declaredoptions", opt.to_string())?;
  let cs_proto = format!("\\ds@{opt}");
  wire_primitive(engine, ast, &cs_proto, body, PrimitiveOptions::default())
}

/// Install a string-template `DefConstructor` as a native constructor. The
/// template is parsed into the shared `ReplacementOp` AST once and interpreted by
/// the core runtime — no Rhai involved per invocation, so this path is fast.
fn wire_constructor_template(proto: &str, template: String) -> Result<()> {
  ConstructorBuilder::new(proto)?
    .replacement(template_replacement(&template)?)
    .install()
}

/// Marshal a digested macro argument into a Rhai value. Every argument is passed
/// as its TeX-source string (authors parse it in-script as needed).
fn arg_to_dynamic(arg: ArgWrap) -> Dynamic {
  Dynamic::from(arg.to_string())
}

/// Read a Rhai return value back as a string (the TeX-source of the expansion).
fn dynamic_to_string(d: Dynamic) -> String {
  if d.is_string() {
    d.into_string().unwrap_or_default()
  } else {
    d.to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use latexml_core::gullet;
  use latexml_core::state::{set_state, State, StateOptions};

  /// Bootstrap enough engine to validate prototypes (`{}` etc. need the base
  /// parameter-type registry). In a real conversion this is already loaded.
  fn fresh_state() {
    set_state(State::new(StateOptions::default()));
    latexml_core::stomach::initialize_stomach();
    latexml_engine::base::load_definitions().expect("bootstrap base parameter types");
  }

  /// The Wave-A pool surface (state, Let/RawTeX, counters, token helpers):
  /// every registration must round-trip through a real script execution.
  #[test]
  fn pool_surface_state_counters_tokens() {
    fresh_state();
    load_script(
      r##"
      AssignValue("ws:k", "v1");
      assign_global("ws:str", LookupString("ws:k"));

      RawTeX("\\def\\wsfoo{FOO}");
      Let("\\wsalias", "\\wsfoo");
      assign_global("ws:def", if IsDefined("\\wsfoo") { "yes" } else { "no" });
      assign_global("ws:alias", if IsDefined("\\wsalias") { "yes" } else { "no" });
      assign_global("ws:xeq", if XEquals("\\wsalias", "\\wsfoo") { "eq" } else { "ne" });
      assign_global("ws:expand", UnTeX(Expand(TokenizeInternal("\\wsfoo"))));

      NewCounter("wsctr");
      StepCounter("wsctr");
      StepCounter("wsctr");
      AddToCounter("wsctr", 3);
      assign_global("ws:cv", CounterValue("wsctr").to_string());
      let refmap = RefStepCounter("wsctr");
      assign_global("ws:ref", if ("tags" in refmap) && ("id" in refmap) { "has" } else { "none" });
      ResetCounter("wsctr");
      assign_global("ws:cv0", CounterValue("wsctr").to_string());

      assign_global("ws:digest", ToString(DigestText("ab")));
    "##,
    )
    .expect("wave-A surface script should load cleanly");
    assert_eq!(lookup_str("ws:str"), "v1", "AssignValue/LookupString");
    assert_eq!(lookup_str("ws:def"), "yes", "RawTeX \\def + IsDefined");
    assert_eq!(lookup_str("ws:alias"), "yes", "Let installs the alias");
    assert_eq!(lookup_str("ws:xeq"), "eq", "XEquals alias == \\wsfoo");
    assert_eq!(lookup_str("ws:expand"), "FOO", "Expand through the gullet");
    assert_eq!(lookup_str("ws:cv"), "5", "2 steps + 3 = 5");
    assert_eq!(lookup_str("ws:ref"), "has", "RefStepCounter returns tags+id");
    assert_eq!(lookup_str("ws:cv0"), "0", "ResetCounter zeroes");
    assert_eq!(lookup_str("ws:digest"), "ab", "DigestText -> Digested handle");
  }

  /// Wave-B definition forms: DefRegister (count + dimen), DefConditional
  /// (Rhai test driven from real TeX), DefKeyVal, DefLigature, DefMath.
  #[test]
  fn pool_surface_definition_forms() {
    fresh_state();
    load_script(
      r##"
      DefRegister("\\wbcount", 42);
      DefRegister("\\wbdimen", "5pt");
      DefKeyVal("WB", "color", "");
      DefLigature("ff", "F");
      DefMath("\\wbsum", "∑", #{ role: "SUMOP", meaning: "sum" });
      DefConditional("\\ifwb{}", |x| x == "on");
      DefMacro("\\wbprobe{}", |x| "\\ifwb{" + x + "}YES\\else NO\\fi");
    "##,
    )
    .expect("wave-B surface script should load cleanly");
    // Registers installed and readable through the native register store.
    assert!(
      latexml_core::state::lookup_definition(&latexml_core::T_CS!("\\wbcount"))
        .expect("lookup")
        .is_some(),
      "\\wbcount register installed"
    );
    assert!(
      latexml_core::state::lookup_definition(&latexml_core::T_CS!("\\wbdimen"))
        .expect("lookup")
        .is_some(),
      "\\wbdimen register installed"
    );
    assert!(
      latexml_core::state::lookup_definition(&latexml_core::T_CS!("\\wbsum"))
        .expect("lookup")
        .is_some(),
      "DefMath \\wbsum installed"
    );
    // The conditional drives real expansion: \ifwb{on} -> YES, \ifwb{off} -> NO.
    let on = latexml_core::gullet::do_expand(mouth::tokenize_internal("\\wbprobe{on}"))
      .expect("expand on");
    assert_eq!(on.to_string().trim(), "YES", "conditional true branch");
    let off = latexml_core::gullet::do_expand(mouth::tokenize_internal("\\wbprobe{off}"))
      .expect("expand off");
    assert_eq!(off.to_string().trim(), "NO", "conditional false branch");
  }

  fn lookup_str(key: &str) -> String {
    match latexml_core::state::lookup_value(key) {
      Some(Stored::String(s)) => arena::to_string(s),
      _ => String::new(),
    }
  }

  /// Conformance: the *same* `afterDigest` constructor defined two ways —
  /// macro-style (calling `ConstructorBuilder` directly, as `DefConstructor!`
  /// lowers) and via Rhai (which now also routes through `ConstructorBuilder`) —
  /// produces identical behaviour. This is the anti-drift guard between
  /// `setup_binding_language.rs` and the Rhai layer.
  #[test]
  fn builder_conformance_macro_style_vs_rhai_afterdigest() {
    use latexml_core::binding::def::builder::{ConstructorBuilder, OptionValue};

    fresh_state();

    // (1) Macro-style: build \mfoo via ConstructorBuilder; afterDigest is a
    // native Rust closure reading the whatsit's first arg.
    let after: DigestionClosure = Rc::new(|w: &mut Whatsit| -> Result<Vec<Digested>> {
      let s = match w.get_arg(1) {
        Some(d) => d.untex()?,
        None => String::new(),
      };
      latexml_core::state::assign_value("conf:m", s, Some(Scope::Global));
      Ok(Vec::new())
    });
    ConstructorBuilder::new("\\mfoo{}")
      .expect("builder")
      .replacement(template_replacement("<ltx:text>#1</ltx:text>").expect("template"))
      .set_option("mode", OptionValue::Str("text".to_string()))
      .expect("set_option")
      .after_digest(after)
      .install()
      .expect("install");
    latexml_core::stomach::digest(mouth::tokenize_internal(r"\mfoo{ZED}")).expect("digest mfoo");

    // (2) Rhai: the equivalent \rfoo — same builder under the hood; afterDigest
    // reads the whatsit via whatsit().
    load_script(
      r#"DefConstructor("\\rfoo{}", "<ltx:text>#1</ltx:text>", #{
           mode: "text",
           afterDigest: || { assign_global("conf:r", whatsit().argString(1)); }
         });"#,
    )
    .expect("load");
    latexml_core::stomach::digest(mouth::tokenize_internal(r"\rfoo{ZED}")).expect("digest rfoo");

    let m = lookup_str("conf:m");
    let r = lookup_str("conf:r");
    assert_eq!(m, "ZED", "macro-style afterDigest did not capture the arg");
    assert_eq!(m, r, "macro-style and Rhai afterDigest diverged: {m:?} vs {r:?}");
    latexml_core::reset_thread_engine();
  }

  #[test]
  fn m1_script_macro_expands_through_real_gullet() {
    fresh_state();
    let n = load_script(
      r#"
        DefMacro("\\twice{}", |x| x + x);
        DefMacro("\\greet{}", |name| "Hello, " + name + "!");
      "#,
    )
    .expect("load_script");
    assert_eq!(n, 2);

    let out = gullet::do_expand(mouth::tokenize_internal(r"\twice{ab}")).expect("expand twice");
    assert_eq!(out.to_string(), "abab");

    let out = gullet::do_expand(mouth::tokenize_internal(r"\greet{World}")).expect("expand greet");
    assert_eq!(out.to_string(), "Hello, World!");

    latexml_core::reset_thread_engine();
  }

  #[test]
  fn m1_expansion_to_control_sequence_is_faithful() {
    fresh_state();
    load_script(r#"DefMacro("\\emphx{}", |x| "\\textit{" + x + "}");"#).expect("load");
    let out = gullet::do_expand(mouth::tokenize_internal(r"\emphx{hi}")).expect("expand");
    assert_eq!(out.to_string(), r"\textit{hi}");
    latexml_core::reset_thread_engine();
  }

  #[test]
  fn cache_reuses_compiled_script_and_still_wires() {
    fresh_state();
    let src = r#"DefMacro("\\dup{}", |x| x + x);"#;
    assert_eq!(load_script(src).expect("first load"), 1);
    // Second load is a cache hit (no recompile) but still installs the binding.
    assert_eq!(load_script(src).expect("second load"), 1);
    let out = gullet::do_expand(mouth::tokenize_internal(r"\dup{yo}")).expect("expand");
    assert_eq!(out.to_string(), "yoyo");
    latexml_core::reset_thread_engine();
  }

  #[test]
  fn load_file_reads_and_installs() {
    fresh_state();
    let path = std::env::temp_dir().join("lx_script_bindings_load_file_test.sty.rhai");
    std::fs::write(&path, r#"DefMacro("\\trip{}", |x| x + x + x);"#).expect("write temp");
    let n = load_file(path.to_str().unwrap()).expect("load_file");
    assert_eq!(n, 1);
    let out = gullet::do_expand(mouth::tokenize_internal(r"\trip{ab}")).expect("expand");
    assert_eq!(out.to_string(), "ababab");
    let _ = std::fs::remove_file(&path);
    latexml_core::reset_thread_engine();
  }

  /// Translation of the Perl doc example:
  ///   DeclareOption('opt', sub { Digest(Tokenize('\relax')); });
  /// We add a marker assignment so the test can observe the body ran.
  #[test]
  fn declare_option_registers_and_runs() {
    fresh_state();
    load_script(
      r#"DeclareOption("opt", || {
           Digest(Tokenize("\\relax"));
           assign_global("script:opt_ran", "yes");
         });"#,
    )
    .expect("load");
    // Invoke the option by digesting its generated \ds@opt primitive.
    latexml_core::stomach::digest(mouth::tokenize_internal(r"\ds@opt")).expect("digest \\ds@opt");
    let ran = match latexml_core::state::lookup_value("script:opt_ran") {
      Some(Stored::String(s)) => arena::to_string(s),
      _ => String::new(),
    };
    assert_eq!(ran, "yes", "DeclareOption body (Tokenize+Digest) did not run");
    latexml_core::reset_thread_engine();
  }

  /// The `DefConstructor` option-bag form: a trailing Rhai map `#{ … }` with
  /// named options (any order, omittable) including a closure-valued
  /// `afterDigest` — the analog of the macro's `key => value` options.
  #[test]
  fn constructor_options_map_runs_afterdigest() {
    fresh_state();
    // The parameterless afterDigest body reaches the in-flight whatsit via
    // whatsit() — referencing context only when needed ("omit as implied").
    load_script(
      r#"DefConstructor("\\opt{}", "<ltx:text>#1</ltx:text>", #{
           mode: "text",
           afterDigest: || { assign_global("script:cad", whatsit().argString(1)); }
         });"#,
    )
    .expect("load");
    latexml_core::stomach::digest(mouth::tokenize_internal(r"\opt{HELLO}")).expect("digest \\opt");
    let ran = match latexml_core::state::lookup_value("script:cad") {
      Some(Stored::String(s)) => arena::to_string(s),
      _ => String::new(),
    };
    assert_eq!(ran, "HELLO", "afterDigest body did not read the whatsit arg via whatsit()");
    latexml_core::reset_thread_engine();
  }

  #[test]
  fn m1_errors_are_clean() {
    fresh_state();
    assert!(load_script("DefMacro(\"\\\\x{}\", |a| a +").is_err());

    fresh_state();
    load_script(r#"DefMacro("\\boom{}", |x| { throw "kaboom"; });"#).expect("load");
    let r = gullet::do_expand(mouth::tokenize_internal(r"\boom{x}"));
    assert!(r.is_err(), "throwing body should error, got {r:?}");
    latexml_core::reset_thread_engine();
  }
}
