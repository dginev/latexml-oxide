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
//! * **`DefMacro`** (expandable) — body receives Perl's `($gullet, @args)`, the arguments as Tokens, and returns
//!   Tokens or TeX source (re-tokenized via `mouth::tokenize_internal`, so `\textit{x}` yields a control sequence).
//! * **`DefConstructor`** (construction) — two forms. A string XML template, or an imperative body
//!   that reads like the Perl original: it receives a `Document` **proxy** as its first argument
//!   (Perl's `$_[0]`), each digested argument as an opaque handle, then Perl's `%props` as one map, e.g. `|document, x| {
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
    options::{
      CONDITIONAL_OPTIONS, CONSTRUCTOR_OPTIONS, ENVIRONMENT_OPTIONS, MACRO_OPTIONS, MATH_OPTIONS,
      PRIMITIVE_OPTIONS, check_options, unsupported_option,
    },
    replacement,
  },
  common::{
    arena::{self, SymHashMap},
    def_parser::parse_prototype,
    dimension::Dimension,
    error::{Error, Result},
    float::Float,
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
  document::{Document, tag::TagConstructionClosure},
  mouth,
  state::Scope,
  token::{Catcode, Token},
  tokens::{TeXString, Tokens},
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

  /// Active-context stack for whatsit-receiving hook closures (`afterDigest`, …): the in-flight whatsit their
  /// `whatsit` argument (and the deprecated `whatsit()`) resolves.
  /// Entries are (whatsit pointer, mutable?). Digestion hooks (`afterDigest`,
  /// `properties`-adjacent) publish mutable whatsits; construction hooks
  /// (`before/afterConstruct`, `reversion`, `sizer`) publish READ-ONLY ones —
  /// `setProperty` errors there instead of mutating through a shared ref.
  static WHATSIT_CTX: RefCell<Vec<(*mut Whatsit, bool)>> = const { RefCell::new(Vec::new()) };

  /// The deprecated aliases already Warned, by script (its AST) — once per binding load; `load_script` clears its own.
  static DEPRECATED_WARNED: RefCell<Vec<(usize, &'static str)>> =
    const { RefCell::new(Vec::new()) };

  /// The already-logged Fatal a registered function returned into Rhai, which can only carry its text (kept
  /// beside it): the script's caller returns it as raised, so it prints once (#792).
  static RAISED_FATAL: RefCell<Option<(Error, String)>> = const { RefCell::new(None) };
}

/// The logged Fatal a registered function raised, if the script's failure `e` is that Fatal; always taken, so a
/// Fatal the script caught never stands in for a later, unrelated error (see `RAISED_FATAL`). Matched by text: a
/// later `throw` that repeats the caught Fatal's whole internal message is taken for it (only a hand-written copy).
pub(super) fn take_raised_fatal(e: &EvalAltResult) -> Option<Error> {
  let (fatal, text) = RAISED_FATAL.with(|r| r.borrow_mut().take())?;
  let raised = match e.unwrap_inner() {
    EvalAltResult::ErrorRuntime(v, _) => v.is_string() && v.to_string() == text,
    _ => false,
  };
  (raised && fatal.is_logged_fatal()).then_some(fatal)
}

/// The names #716 replaces with Perl's forms, kept one release as aliases: the name a script writes and its
/// replacement. [`warn_deprecated`] names both; `api_doc` marks each row.
pub(super) const DEPRECATED_ALIASES: &[(&str, &str)] = &[
  ("whatsit", "the hook's whatsit argument"),
  ("whatsit.argString", "whatsit.getArg(n)"),
  ("whatsit.propertyString", "whatsit.getProperty(key)"),
  (
    "document.getProperty",
    "the constructor body's trailing properties map",
  ),
  (
    "document.getProperties",
    "the constructor body's trailing properties map",
  ),
  ("assign_value", "AssignValue(key, value, \"local\")"),
  ("assign_global", "AssignValue(key, value, \"global\")"),
  ("lookup_value", "LookupValue(key)"),
  ("AssignString", "AssignValue(key, value[, scope])"),
  ("AssignNumber", "AssignValue(key, value[, scope])"),
  ("AssignBool", "AssignValue(key, value[, scope])"),
  ("AssignMeaning", "Let(cs, other[, scope])"),
  (
    "DefAccent",
    "DefAccent(accent, combining, standalone, #{below: true})",
  ),
  ("DefMacro |args|", "|gullet, args|, the arguments as Tokens"),
  (
    "DefPrimitive |args|",
    "|stomach, args|, the arguments as Tokens",
  ),
  (
    "DefColumnType |args|",
    "|gullet, args|, the arguments as Tokens",
  ),
  ("reversion |args|", "|whatsit, args|"),
];

/// One `Warning:deprecated:<old>` per binding load for a deprecated alias, naming its replacement.
pub(super) fn warn_deprecated(old: &'static str) {
  let script = CURRENT_SCRIPT.with(|c| {
    c.borrow()
      .last()
      .map_or(0, |(_, ast)| Rc::as_ptr(ast) as usize)
  });
  let first = DEPRECATED_WARNED.with(|w| {
    let mut warned = w.borrow_mut();
    let first = !warned.contains(&(script, old));
    if first {
      warned.push((script, old));
    }
    first
  });
  if first {
    let new = DEPRECATED_ALIASES
      .iter()
      .find(|(name, _)| *name == old)
      .map_or("", |(_, new)| *new);
    latexml_core::Warn!("deprecated", old, format!("{old} is deprecated; use {new}"));
  }
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
  let mut args_vec: Vec<Dynamic> = Vec::new();
  args.parse(&mut args_vec);
  // A Perl sub takes any number of arguments: a closure's missing ones are `()` (undef), extra ones dropped.
  // A closure's captured variables are curried ahead of its own parameters.
  if let Some(params) = closure_arity(ast, body.fn_name()) {
    args_vec.resize(params.saturating_sub(body.curry().len()), Dynamic::UNIT);
  }
  body.call::<Dynamic>(engine, ast, args_vec)
}

/// [`closure_arity`]'s entry: the AST it was read from (held weakly: a dropped script's entry is recomputed) and
/// the arity.
type ArityEntry = (std::rc::Weak<AST>, Option<usize>);

thread_local! {
  static ARITY_CACHE: RefCell<HashMap<(usize, String), ArityEntry>> = RefCell::new(HashMap::new());
}

/// The parameter count of the script function `fn_name` (a closure's `anon$…`), `None` when the name is
/// overloaded; looked up once per AST and name.
fn closure_arity(ast: &Rc<AST>, fn_name: &str) -> Option<usize> {
  let key = (Rc::as_ptr(ast) as usize, fn_name.to_string());
  ARITY_CACHE.with(|cache| {
    if let Some((weak, arity)) = cache.borrow().get(&key)
      && weak.upgrade().is_some_and(|live| Rc::ptr_eq(&live, ast))
    {
      return *arity;
    }
    let mut arities = ast
      .iter_functions()
      .filter(|f| f.name == fn_name)
      .map(|f| f.params.len());
    let arity = match (arities.next(), arities.next()) {
      (Some(params), None) => Some(params),
      _ => None,
    };
    cache.borrow_mut().insert(key, (Rc::downgrade(ast), arity));
    arity
  })
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

/// [`with_doc`] for a query that cannot fail, so the registration returns a plain
/// value instead of a `Result`. Outside a constructor body there is no document
/// to consult, and a script has nothing useful to do with that fact, so the
/// caller's `Default` (an empty array, a `()`) is the answer.
fn with_doc_infallible<R: Default>(f: impl FnOnce(&mut Document) -> R) -> R {
  with_doc(|doc, _props| Ok(f(doc))).unwrap_or_default()
}

/// A Rhai attribute map (`#{ class: "x" }`) as `Document`'s attribute argument.
/// Values stringify, matching every other string-in/string-out point of this API.
fn attribute_map(map: Map) -> rustc_hash::FxHashMap<String, String> {
  map
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

/// Unwrap a script-held node for a document operation that SPLICES it, refusing
/// one that came from `ParseXML`.
///
/// A parsed node is still owned by the throwaway document it was parsed into, so
/// moving it into ours would leave our tree pointing at memory freed the moment
/// the script drops its handle — a use-after-free across FFI, the failure mode
/// that has already cost this project SIGSEGVs and that `catch_unwind` cannot
/// contain. `insertXML` is the supported route for parsed markup: it RE-CREATES
/// the subtree through `append_tree` rather than moving it.
fn in_tree(
  node: NodeProxy,
  method: &str,
) -> std::result::Result<libxml::tree::Node, Box<EvalAltResult>> {
  if node.1.is_some() {
    return Err(Box::<EvalAltResult>::from(format!(
      "{method}: this node came from ParseXML and belongs to its own parsed \
       document; insert it with insertXML (which re-creates it) instead of \
       splicing it in directly"
    )));
  }
  Ok(node.0)
}

/// [`in_tree`] over an array argument, rejecting non-node entries by position —
/// never silently skipping one, which would drop part of what a script asked for.
fn in_tree_array(
  nodes: rhai::Array,
  method: &str,
) -> std::result::Result<Vec<libxml::tree::Node>, Box<EvalAltResult>> {
  let mut out = Vec::with_capacity(nodes.len());
  for (i, entry) in nodes.into_iter().enumerate() {
    let kind = entry.type_name();
    let proxy = entry.try_cast::<NodeProxy>().ok_or_else(|| {
      Box::<EvalAltResult>::from(format!(
        "{method}: entry {i} of the array is a {kind}, not a Node"
      ))
    })?;
    out.push(in_tree(proxy, method)?);
  }
  Ok(out)
}

/// `Document::rename_node` `expect`s a parent — renaming an ORPHAN panics, and a
/// panic from an untrusted script aborts the conversion (the process, under
/// `panic = "abort"`). Check first and report it as an ordinary script error.
fn rename_node_checked(
  node: NodeProxy,
  newname: &str,
  reinsert: bool,
) -> std::result::Result<Dynamic, Box<EvalAltResult>> {
  let node = in_tree(node, "renameNode")?;
  if node.get_parent().is_none() {
    return Err(Box::<EvalAltResult>::from(
      "renameNode: the node has no parent (already removed, or a root); \
       renaming it is not possible",
    ));
  }
  with_doc(|doc, _props| {
    let renamed = doc.rename_node(node, newname, reinsert).map_err(rhai_err)?;
    Ok(Dynamic::from(NodeProxy::borrowed(renamed)))
  })
}

/// A `Document` node-list result as a Rhai array of in-tree node handles.
fn nodes_to_array(nodes: Vec<libxml::tree::Node>) -> rhai::Array {
  nodes
    .into_iter()
    .map(|n| Dynamic::from(NodeProxy::borrowed(n)))
    .collect()
}

/// An optional node as `Node` or `()` — Perl's `undef` for "no match".
fn node_or_unit(node: Option<libxml::tree::Node>) -> Dynamic {
  match node {
    Some(n) => Dynamic::from(NodeProxy::borrowed(n)),
    None => Dynamic::UNIT,
  }
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
/// A node may instead OWN its document, which lifts that restriction for parsed
/// markup: `ParseXML` builds its nodes in a throwaway document, and since libxml's
/// `Document` is refcounted, holding a clone here keeps the node valid for exactly
/// as long as the script keeps the handle. One type serves both provenances on
/// purpose — every node method then works on in-tree and parsed nodes alike, and a
/// new one is a single registration rather than a parallel API.
#[derive(Clone)]
pub(crate) struct NodeProxy(
  pub(crate) libxml::tree::Node,
  /// `Some` when this handle keeps its own (parsed) document alive; `None` for a
  /// node inside the conversion's document, whose lifetime the caller guarantees.
  pub(crate) Option<libxml::tree::Document>,
);

impl NodeProxy {
  /// A node inside the live conversion document — valid only for the body that
  /// received it (see the caveat above).
  pub(crate) fn borrowed(node: libxml::tree::Node) -> Self { Self(node, None) }

  /// A node whose owning document this handle keeps alive.
  pub(crate) fn owning(node: libxml::tree::Node, doc: libxml::tree::Document) -> Self {
    Self(node, Some(doc))
  }

  /// A node reached FROM this one (parent, sibling, child): it lives in the same
  /// document, so it must inherit the same provenance — otherwise walking a
  /// parsed tree would hand back nodes that no longer keep their owner alive.
  pub(crate) fn derived(&self, node: libxml::tree::Node) -> Self { Self(node, self.1.clone()) }
}

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

/// Rhai proxy for the in-flight whatsit, passed to a hook body positionally as Perl passes `$whatsit` (#716). Like
/// `DocProxy`, it carries no pointer; methods resolve the active whatsit context.
#[derive(Clone)]
struct WhatsitProxy;

/// Rhai proxy returned by `LookupDefinition(cs)` (#321). Carries only the CS name;
/// each `push*/unshift*` re-looks-up the CURRENT front definition, splices a hook,
/// and re-installs — so sequential pushes accumulate (mirroring Perl BookML's
/// in-place mutation of the shared def-hash). `LookupDefinition` returns `()`, not
/// this proxy, when the CS is undefined.
#[derive(Clone)]
struct DefinitionProxy {
  cs: Token,
}

/// Resolve the top whatsit active-context (used by `WhatsitProxy` methods).
fn current_whatsit() -> std::result::Result<*mut Whatsit, Box<EvalAltResult>> {
  current_whatsit_entry().map(|(w, _)| w)
}

/// Run `f` on the in-flight whatsit, read-only — the single site that re-mints it for the readers.
fn with_whatsit<R>(f: impl FnOnce(&Whatsit) -> R) -> std::result::Result<R, Box<EvalAltResult>> {
  // SAFETY: `current_whatsit()` returns the in-flight whatsit pointer the core published onto WHATSIT_CTX for the
  // duration of THIS hook body (an error, not UB, outside one); read only, so no aliasing `&mut` exists.
  let w = unsafe { &*current_whatsit()? };
  Ok(f(w))
}

/// The top whatsit entry incl. its mutability flag.
fn current_whatsit_entry() -> std::result::Result<(*mut Whatsit, bool), Box<EvalAltResult>> {
  WHATSIT_CTX.with(|c| {
    c.borrow()
      .last()
      .copied()
      .ok_or_else(|| Box::<EvalAltResult>::from("a whatsit method called outside a hook body"))
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
  let script = Rc::as_ptr(&cached.ast) as usize;
  DEPRECATED_WARNED.with(|w| w.borrow_mut().retain(|(s, _)| *s != script));
  // A dropped script's closure arities go with it.
  ARITY_CACHE.with(|c| c.borrow_mut().retain(|_, (ast, _)| ast.strong_count() > 0));
  let run = {
    let _script_guard = ScriptCtxGuard::new((cached.engine.clone(), cached.ast.clone()));
    cached.engine.run_ast(&cached.ast).map_err(|e| {
      take_raised_fatal(&e).unwrap_or_else(|| Error::from(format!("script-binding run error: {e}")))
    })
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

/// Convert a Rhai object map into a whatsit property map, each value with its type ([`dynamic_to_stored`]); a
/// `Digested` under the `font` key contributes its FONT (the IEEEproof "titlefont" idiom).
fn rhai_map_to_props(map: Map) -> SymHashMap<Stored> {
  let mut props: SymHashMap<Stored> = SymHashMap::default();
  for (k, v) in map {
    match v.clone().try_cast::<Digested>() {
      Some(d) => {
        if k.as_str() == "font"
          && let Ok(Some(f)) = d.get_font()
        {
          props.insert("font", Stored::Font(f));
          continue;
        }
        props.insert(k.as_str(), Stored::Digested(d));
      },
      // A constructor reads a token property as its ToString, as Perl's absorb (Object.pm:169) and attributes do.
      _ if v.is::<Tokens>() || v.is::<Token>() => {
        props.insert(k.as_str(), dynamic_to_string(v).into())
      },
      _ => props.insert(k.as_str(), dynamic_to_stored(v)),
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
/// A latexml `Err` that is a logged Fatal is kept for the script's caller (`take_raised_fatal`).
fn rhai_err<E: std::fmt::Display + 'static>(e: E) -> Box<EvalAltResult> {
  let text = e.to_string();
  if let Ok(err) = (Box::new(e) as Box<dyn std::any::Any>).downcast::<Error>()
    && err.is_logged_fatal()
  {
    RAISED_FATAL.with(|r| *r.borrow_mut() = Some((*err, text.clone())));
  }
  Box::<EvalAltResult>::from(text)
}

/// A token argument: a string names a control sequence (Perl `Let`'s "most common case", Package.pm:385-387), a
/// `Token` is used as is; anything else, a `Tokens` included, is an error naming the call.
fn token_arg(call: &str, v: Dynamic) -> std::result::Result<Token, Box<EvalAltResult>> {
  if v.is_string() {
    Ok(latexml_core::T_CS!(
      v.into_immutable_string().unwrap_or_default().as_str()
    ))
  } else if let Some(t) = v.clone().try_cast::<Token>() {
    Ok(t)
  } else {
    let got = v.type_name();
    Err(
      format!(
        "{call}: expected a control-sequence name or a Token, got {}",
        got.rsplit("::").next().unwrap_or(got)
      )
      .into(),
    )
  }
}

/// A scope string as the macros read it: "local"/"global", "" for the TeX default, any other name a named scope
/// (Perl's `'section:1'`, active only once activated).
fn scope_of(scope: &str) -> Option<Scope> {
  latexml_core::binding::def::traits::IntoOption::into_option(scope)
}

/// Marshal a `Stored` out to a Rhai value with its type (Perl hands back the stored object, Package.pm
/// LookupValue :257, LookupMapping :283): a string, bool, int, float, `Token`, `Tokens` or digested handle as
/// itself, a list value as an array of such (Perl's arrayref, #315), a hash as a map, a char or catcode as its int,
/// `None` as `()` (undef). A register value (Dimension, Glue, …: round C) is its TeX string; a kind with no script
/// form (a node, a font map, a stash) is the placeholder `Stored[??]`. The inverse of [`dynamic_to_stored`].
fn stored_to_dynamic(v: Stored) -> Dynamic {
  match v {
    Stored::None => Dynamic::UNIT,
    Stored::String(s) => Dynamic::from(arena::to_string(s)),
    Stored::Bool(b) => Dynamic::from(b),
    Stored::Int(i) => Dynamic::from(i),
    Stored::Number(n) => Dynamic::from(n.0),
    Stored::Float(f) => Dynamic::from(f.0),
    Stored::Tokens(t) => Dynamic::from(t),
    Stored::Token(t) => Dynamic::from(t),
    Stored::Digested(d) => Dynamic::from(d),
    Stored::Strings(ss) => Dynamic::from_array(
      ss.iter()
        .map(|s| Dynamic::from(arena::to_string(*s)))
        .collect(),
    ),
    Stored::VecDequeStored(items) => {
      Dynamic::from_array(items.into_iter().map(stored_to_dynamic).collect())
    },
    Stored::VecDigested(items) => {
      Dynamic::from_array(items.into_iter().map(Dynamic::from).collect())
    },
    Stored::HashStored(h) => Dynamic::from_map(
      h.0
        .into_iter()
        .map(|(k, v)| (arena::to_string(k).into(), stored_to_dynamic(v)))
        .collect(),
    ),
    Stored::HashString(h) => Dynamic::from_map(
      h.into_iter()
        .map(|(k, v)| (k.into(), Dynamic::from(v)))
        .collect(),
    ),
    Stored::Charcode(c) => Dynamic::from(i64::from(c)),
    Stored::Catcode(c) => Dynamic::from(c as i64),
    other => Dynamic::from(other.to_string()),
  }
}

/// Marshal a Rhai value INTO a `Stored` (Perl's value table is untyped, Package.pm AssignValue :261,
/// AssignMapping :287, PushValue): string, int, float, bool, `Token`, `Tokens` and a digested handle keep their
/// type, an array is a list and a map a hash of such (Perl's arrayref/hashref), `()` is `None` (undef); anything
/// else falls back to its string form (#540). The inverse of [`stored_to_dynamic`].
fn dynamic_to_stored(v: Dynamic) -> Stored {
  if v.is_unit() {
    Stored::None
  } else if v.is_string() {
    v.into_string().unwrap_or_default().into()
  } else if let Ok(i) = v.as_int() {
    Stored::Int(i)
  } else if let Ok(f) = v.as_float() {
    Stored::Float(Float(f))
  } else if let Ok(b) = v.as_bool() {
    Stored::Bool(b)
  } else if let Some(tokens) = v.clone().try_cast::<Tokens>() {
    Stored::Tokens(tokens)
  } else if let Some(token) = v.clone().try_cast::<Token>() {
    Stored::Token(token)
  } else if let Some(digested) = v.clone().try_cast::<Digested>() {
    Stored::Digested(digested)
  } else if v.is_array() {
    Stored::VecDequeStored(
      v.cast::<rhai::Array>()
        .into_iter()
        .map(dynamic_to_stored)
        .collect(),
    )
  } else if v.is_map() {
    let mut hash = SymHashMap::default();
    for (k, v) in v.cast::<Map>() {
      hash.insert(k.as_str(), dynamic_to_stored(v));
    }
    Stored::HashStored(hash)
  } else {
    v.to_string().into()
  }
}

/// A closure's returned expansion or reversion as Tokens: a `Token` or `Tokens` as itself (a Perl macro sub
/// returns tokens), `()` as nothing, a string (the Rhai shorthand) TokenizeInternal'd as before.
fn dynamic_to_tokens(d: Dynamic) -> Tokens {
  if let Some(tokens) = d.clone().try_cast::<Tokens>() {
    tokens
  } else if let Some(token) = d.clone().try_cast::<Token>() {
    Tokens::new(vec![token])
  } else if d.is_unit() {
    Tokens::default()
  } else {
    mouth::tokenize_internal(TeXString::assembled(dynamic_to_string(d)))
  }
}

/// An argument as its TeX-source string: an old-form body's (deprecated), a `properties` closure's.
fn arg_to_dynamic(arg: ArgWrap) -> Dynamic { Dynamic::from(arg.to_string()) }

/// Read a Rhai value back as a string, Perl's `ToString` (Object.pm:59): a `Token`/`Tokens` as `Tokens->toString`
/// (`\foo bar` → `\foobar`, as [`rhai_map_to_props`] stores a token property), a digested handle its `ToString`
/// (Rhai's own printing gives a custom type's Rust name).
fn dynamic_to_string(d: Dynamic) -> String {
  if d.is_string() {
    d.into_string().unwrap_or_default()
  } else if let Some(tokens) = d.clone().try_cast::<Tokens>() {
    tokens.to_string()
  } else if let Some(token) = d.clone().try_cast::<Token>() {
    token.to_string()
  } else if let Some(digested) = d.clone().try_cast::<Digested>() {
    digested.to_string()
  } else {
    d.to_string()
  }
}

/// The complete `.rhai` binding interface, rendered into this crate's rustdoc.
///
/// Not hand-written: `API.md` is GENERATED from the live engine (Rhai's
/// `gen_fn_signatures` reflections API) and a test regenerates and compares it on
/// every run, so `cargo doc` — and docs.rs — always show what is actually
/// registered. Regenerate after adding a `register_fn` with:
///
/// ```sh
/// UPDATE_API_DOC=1 cargo test -p latexml_contrib --lib api_reference_is_up_to_date
/// ```
// The table links each call as `[`short_name`](full::path)`. `short_name` is
// what a reader can scan; the full path is what rustdoc resolves. Where a short
// name ALSO happens to resolve on its own — `def_macro` does, because
// `#[macro_use] extern crate` pulls the engine and codegen macro namespaces in
// crate-wide — rustdoc calls the explicit target redundant. It is not wrong,
// and the link works; requiring the generator to drop the target on exactly the
// rows where the label collides would make the output depend on what happens to
// be in scope. `broken_intra_doc_links` — the lint that catches a DEAD link,
// and the reason `cargo doc` is gated on `-D warnings` at all — stays on.
#[allow(rustdoc::redundant_explicit_links)]
#[doc = include_str!("API.md")]
pub mod interface {}

#[cfg(test)]
mod api_doc;
#[cfg(test)]
mod tests;
