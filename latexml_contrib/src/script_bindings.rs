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
use latexml_core::binding::def::dialect::{def_macro, def_primitive};
use latexml_core::binding::def::replacement;
use latexml_core::common::arena::{self, SymHashMap};
use latexml_core::common::def_parser::parse_prototype;
use latexml_core::common::error::{Error, Result};
use latexml_core::common::store::Stored;
use latexml_core::definition::argument::ArgWrap;
use latexml_core::definition::primitive::PrimitiveOptions;
use latexml_core::definition::{
  BeforeDigestClosure, DigestionClosure, ExpansionBody, ExpansionClosure, PrimitiveBody,
  PrimitiveClosure, PropertiesClosure, ReplacementClosure,
};
use latexml_core::digested::Digested;
use latexml_core::document::Document;
use latexml_core::mouth;
use latexml_core::state::Scope;
use latexml_core::tokens::Tokens;
use latexml_core::whatsit::Whatsit;
use latexml_core::BoxOps;

// Sandbox limits (docs/script_bindings_plan.md §6).
const MAX_OPERATIONS: u64 = 50_000_000;
const MAX_CALL_LEVELS: usize = 128;
const MAX_STRING_SIZE: usize = 4 * 1024 * 1024;

/// A registration collected while a script runs. `Clone` so it can be cached
/// and re-wired into successive conversions' States without recompiling.
/// A constructor's replacement — either an XML template or an imperative body.
#[derive(Clone)]
enum ConstructorRepl {
  Template(String),
  Closure(FnPtr),
}

#[derive(Clone)]
enum Reg {
  Macro(String, FnPtr),
  Primitive(String, FnPtr),
  Constructor(String, FnPtr),
  ConstructorTemplate(String, String),
  /// `DefConstructor(proto, replacement, #{ options })` — the option-bag form,
  /// mirroring the `DefConstructor!` macro's variadic `key => value` options.
  ConstructorOpts(String, ConstructorRepl, Map),
  /// `DefEnvironment(proto, replacement[, #{ options }])` — all four shapes
  /// (template/closure × with/without options) collapse here (empty `Map` when
  /// no options were given).
  Environment(String, ConstructorRepl, Map),
  Option(String, FnPtr),
}

/// A compiled-and-collected script, cached by source so the (relatively
/// expensive) Rhai compile + run happens once per unique binding even when the
/// same contrib package is loaded across many conversions.
#[derive(Clone)]
struct CachedScript {
  engine: Rc<Engine>,
  ast: Rc<AST>,
  regs: Vec<Reg>,
}

thread_local! {
  static REGS: RefCell<Vec<Reg>> = const { RefCell::new(Vec::new()) };

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

  // ── registration API (collected, wired to native defs after the run) ──
  engine.register_fn("DefMacro", |proto: &str, body: FnPtr| {
    REGS.with(|m| m.borrow_mut().push(Reg::Macro(proto.to_string(), body)));
  });
  engine.register_fn("DefPrimitive", |proto: &str, body: FnPtr| {
    REGS.with(|m| m.borrow_mut().push(Reg::Primitive(proto.to_string(), body)));
  });
  // Class/package option. Mirrors the `DeclareOption!` macro's lowering
  // (setup_binding_language.rs): push the name onto `@declaredoptions` and
  // define a `\ds@<opt>` primitive carrying the body.
  engine.register_fn("DeclareOption", |opt: &str, body: FnPtr| {
    REGS.with(|m| m.borrow_mut().push(Reg::Option(opt.to_string(), body)));
  });

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
  engine.register_fn("DefConstructor", |proto: &str, body: FnPtr| {
    REGS.with(|m| m.borrow_mut().push(Reg::Constructor(proto.to_string(), body)));
  });
  // String-template form (the dominant constructor dialect): the second arg is
  // an XML template instead of a closure. Rhai dispatches by the arg's type.
  engine.register_fn("DefConstructor", |proto: &str, template: &str| {
    REGS.with(|m| {
      m.borrow_mut()
        .push(Reg::ConstructorTemplate(proto.to_string(), template.to_string()))
    });
  });
  // Option-bag forms: a trailing Rhai object map `#{ mode: …, afterDigest: |…| … }`
  // — the analog of Perl's `%options` / the `DefConstructor!` macro's `key => value`
  // (named, any order, omittable; values may be strings or closures).
  engine.register_fn("DefConstructor", |proto: &str, template: &str, opts: Map| {
    REGS.with(|m| {
      m.borrow_mut().push(Reg::ConstructorOpts(
        proto.to_string(),
        ConstructorRepl::Template(template.to_string()),
        opts,
      ))
    });
  });
  engine.register_fn("DefConstructor", |proto: &str, body: FnPtr, opts: Map| {
    REGS.with(|m| {
      m.borrow_mut().push(Reg::ConstructorOpts(
        proto.to_string(),
        ConstructorRepl::Closure(body),
        opts,
      ))
    });
  });

  // ── DefEnvironment: same four shapes as DefConstructor; the prototype is the
  // `DefEnvironment!` form (`"{name}"` / `"{name}{}…"`), the template will
  // typically reference `#body`. ──
  engine.register_fn("DefEnvironment", |proto: &str, template: &str| {
    REGS.with(|m| {
      m.borrow_mut().push(Reg::Environment(
        proto.to_string(),
        ConstructorRepl::Template(template.to_string()),
        Map::new(),
      ))
    });
  });
  engine.register_fn("DefEnvironment", |proto: &str, body: FnPtr| {
    REGS.with(|m| {
      m.borrow_mut().push(Reg::Environment(
        proto.to_string(),
        ConstructorRepl::Closure(body),
        Map::new(),
      ))
    });
  });
  engine.register_fn("DefEnvironment", |proto: &str, template: &str, opts: Map| {
    REGS.with(|m| {
      m.borrow_mut().push(Reg::Environment(
        proto.to_string(),
        ConstructorRepl::Template(template.to_string()),
        opts,
      ))
    });
  });
  engine.register_fn("DefEnvironment", |proto: &str, body: FnPtr, opts: Map| {
    REGS.with(|m| {
      m.borrow_mut().push(Reg::Environment(
        proto.to_string(),
        ConstructorRepl::Closure(body),
        opts,
      ))
    });
  });

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

/// Load a binding script: compile + run once (cached by source), then install a
/// native definition for each registration into the current State. Returns the
/// number of bindings installed.
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
      REGS.with(|m| m.borrow_mut().clear());
      engine
        .run_ast(&ast)
        .map_err(|e| Error::from(format!("script-binding run error: {e}")))?;
      let regs: Vec<Reg> = REGS.with(|m| m.borrow_mut().drain(..).collect());
      let cs = CachedScript { engine: Rc::new(engine), ast: Rc::new(ast), regs };
      SCRIPT_CACHE.with(|c| {
        c.borrow_mut().insert(src.to_string(), cs.clone());
      });
      cs
    },
  };

  // Wire each registration into the current State (cheap; done per conversion).
  let count = cached.regs.len();
  for reg in &cached.regs {
    match reg {
      Reg::Macro(proto, body) => wire_macro(&cached.engine, &cached.ast, proto, body.clone())?,
      Reg::Primitive(proto, body) => {
        wire_primitive(&cached.engine, &cached.ast, proto, body.clone())?
      },
      Reg::Constructor(proto, body) => {
        wire_constructor(&cached.engine, &cached.ast, proto, body.clone())?
      },
      Reg::ConstructorTemplate(proto, tmpl) => wire_constructor_template(proto, tmpl.clone())?,
      Reg::ConstructorOpts(proto, repl, opts) => {
        wire_constructor_opts(&cached.engine, &cached.ast, proto, repl.clone(), opts.clone())?
      },
      Reg::Environment(proto, repl, opts) => {
        wire_environment(&cached.engine, &cached.ast, proto, repl.clone(), opts.clone())?
      },
      Reg::Option(opt, body) => wire_option(&cached.engine, &cached.ast, opt, body.clone())?,
    }
  }
  Ok(count)
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
  fn before_digest(self, hook: BeforeDigestClosure) -> Self;
  fn properties(self, props: PropertiesClosure) -> Self;
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
      fn before_digest(self, hook: BeforeDigestClosure) -> Self {
        <$t>::before_digest(self, hook)
      }
      fn properties(self, props: PropertiesClosure) -> Self { <$t>::properties(self, props) }
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
        "properties" => {
          builder = builder.properties(properties_trampoline(fp, engine.clone(), ast.clone()));
        },
        "beforeDigest" => {
          builder = builder.before_digest(before_digest_trampoline(fp, engine.clone(), ast.clone()));
        },
        // Unknown closure options are silently ignored (forgiving, like Perl
        // %options; the builder's set_option does the same for scalars).
        _ => {},
      }
    } else if key.as_str() == "properties" && val.is_map() {
      // Static property map (Perl's `properties => { key => value, … }`).
      let map = val.cast::<Map>();
      builder = builder.properties(Rc::new(move |_args| Ok(rhai_map_to_props(map.clone()))));
    } else if let Some(ov) = dynamic_to_option_value(&val) {
      // Scalar option → the builder's generic, single-source `set_option`.
      builder = builder.set_option(key.as_str(), ov)?;
    }
  }
  Ok(builder)
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

/// Convert a Rhai object map into a whatsit property map (string values).
fn rhai_map_to_props(map: Map) -> SymHashMap<Stored> {
  let mut props: SymHashMap<Stored> = SymHashMap::default();
  for (k, v) in map {
    props.insert(k.as_str(), dynamic_to_string(v).into());
  }
  props
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
fn wire_primitive(engine: &Rc<Engine>, ast: &Rc<AST>, proto: &str, body: FnPtr) -> Result<()> {
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

  def_primitive(cs, paramlist, Some(PrimitiveBody::Closure(closure)), PrimitiveOptions::default())?;
  Ok(())
}

/// Install a `DeclareOption` registration. Mirrors the `DeclareOption!` macro
/// (`setup_binding_language.rs`): record the option name in `@declaredoptions`
/// and define a `\ds@<opt>` primitive whose body runs when the option fires.
fn wire_option(engine: &Rc<Engine>, ast: &Rc<AST>, opt: &str, body: FnPtr) -> Result<()> {
  latexml_core::state::push_value("@declaredoptions", opt.to_string())?;
  let cs_proto = format!("\\ds@{opt}");
  wire_primitive(engine, ast, &cs_proto, body)
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
