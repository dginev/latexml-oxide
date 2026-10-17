//! Definition provenance — K1 of the generalized kernel-capability program
//! (`docs/perfect_kernel/KERNEL_CAPABILITIES.md`).
//!
//! Every definition records WHO made it: LaTeXML's own layers (the format
//! dumps, the Rust pools, a compiled binding, the raw format files) or the
//! document's world (a raw `.sty`/`.cls`/`.def` the document loaded, or the
//! document itself). The origin is captured at construction from a
//! thread-local set by the loader seams, the same moment a definition's
//! locator is, and travels with the object through `\let`.
//!
//! Its first consumer is the l3/ltcmd "already defined" leniency
//! (`\lx@if@pooldefined`): real LaTeX keeps `\section`, `\thepage`,
//! `{figure}`… in article.cls, our pool pre-defines them for class-less
//! robustness, so a standalone class redeclaring them is not an error —
//! while a genuine double declaration between two raw files still is.
use std::cell::{Cell, RefCell};

/// Who made a definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionOrigin {
  /// Made before any loader seam set an origin (engine bootstrap) — ours.
  Unknown,
  /// The PLAIN layer: restored from the plain format dump (embedded or a
  /// `plain.YYYY.dump.txt` on disk), or made by a plain pool (`plain_*`). Perl
  /// tells the same layer by its definitions' source (`isDefinableLaTeX`,
  /// latex_constructs.pool.ltxml:2512-2517: `getShortSource =~ /^plain/`): LaTeX's
  /// `\newcommand`/`\@ifdefinable` may redefine what plain defined.
  Plain,
  /// Restored from the LaTeX format dump (`<embedded:latex>`).
  LatexDump,
  /// A Rust pool (`InnerPool!`/`LoadPool!`: TeX.pool, LaTeX.pool, constructs).
  Pool,
  /// A compiled binding's own code (`DefMacro!`/`RawTeX!` in a `*_sty.rs`).
  Binding,
  /// The guess of a fallback class — OmniBus, loaded for a class with no binding: what it defines is a
  /// convenience the real class may not have, so a document's own `\newcommand` (and `\providecommand`,
  /// `\newenvironment`, `\@ifdefinable`) of the name wins over it, unless it is locked
  /// (`state::is_fallback_meaning`).
  Fallback,
  /// A kernel stand-in for a name a class or style defines only where the document cannot reach it (aaai's
  /// `\equalcontrib`, set inside its `\@maketitle`): yields as `Fallback` does, and to a raw class's or package's own
  /// `\newcommand` of the name too, since the real file does define it (copernicus.cls:1673 `\equalcontrib[1]`).
  Stub,
  /// What a binding borrows from a package the real one does not load (`binding::content::require_package_borrowed`:
  /// theorem.sty's and ntheorem.sty's bindings borrow amsthm's): in LaTeX the names do not exist, so they yield as
  /// `Fallback` does, and to every raw file's own definition too — TeX Live's or the author's (cc.cls's ccproof.sty:56
  /// `\newenvironment{proof}` after theorem.sty; 1507.02955).
  Borrowed,
  /// A raw format file read as definitions (latex.ltx, plain.tex, expl3-code.tex).
  Format,
  /// A raw `.sty`/`.cls`/`.def`/`.tex` the document loaded as definitions.
  File,
  /// The document's own preamble or body.
  Document,
}

impl DefinitionOrigin {
  /// Is this definition LaTeXML's own — a pool, dump, binding or format layer —
  /// rather than the document's? The l3/ltcmd declarators tolerate
  /// redefinition of our own pre-definitions (the class-level names real
  /// LaTeX keeps in article.cls) and report genuine double declarations.
  pub fn is_latexml_owned(self) -> bool {
    !matches!(self, DefinitionOrigin::File | DefinitionOrigin::Document)
  }
}

thread_local! {
  static CURRENT_ORIGIN: Cell<DefinitionOrigin> = const { Cell::new(DefinitionOrigin::Document) };
}

/// The origin a definition constructed now will carry.
pub fn current_origin() -> DefinitionOrigin { CURRENT_ORIGIN.with(|c| c.get()) }

/// RAII guard: definitions made while it lives carry `origin`; the previous
/// origin is restored on drop (loaders nest: a binding raw-loading its `.sty`
/// yields `File` for that file's definitions and `Binding` for its own).
pub struct OriginGuard(DefinitionOrigin);

impl OriginGuard {
  pub fn new(origin: DefinitionOrigin) -> Self {
    OriginGuard(CURRENT_ORIGIN.with(|c| c.replace(origin)))
  }
}

impl Drop for OriginGuard {
  fn drop(&mut self) { CURRENT_ORIGIN.with(|c| c.set(self.0)); }
}

thread_local! {
  static PROVIDE_ONLY: Cell<bool> = const { Cell::new(false) };
}

/// Is a provide-only load in progress ([`ProvideOnlyGuard`])?
pub fn provide_only() -> bool { PROVIDE_ONLY.with(|c| c.get()) }

/// RAII guard: while it lives, a load only provides — a binding's definition (the fallback class OmniBus, or any binding it
/// requires) of a name defined before the guard began is dropped, as a `\providecommand` of it is
/// (`state::provide_only_declines`); the previous mode is restored on drop.
pub struct ProvideOnlyGuard(bool);

impl ProvideOnlyGuard {
  pub fn new() -> Self {
    crate::state::reset_provide_only_assigned();
    ProvideOnlyGuard(PROVIDE_ONLY.with(|c| c.replace(true)))
  }
}

impl Default for ProvideOnlyGuard {
  fn default() -> Self { Self::new() }
}

impl Drop for ProvideOnlyGuard {
  fn drop(&mut self) {
    PROVIDE_ONLY.with(|c| c.set(self.0));
    if !self.0 {
      crate::state::end_provide_only();
    }
  }
}

thread_local! {
  /// The stand-in copy whose provides-mode is in force (`0`: none), and the modes it interrupted.
  static COPY_PROVIDES: Cell<u32> = const { Cell::new(0) };
  static COPY_PROVIDES_STACK: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

/// The stand-in copy whose read is in force — "a stand-in's copy provides" (`state::note_copy_provides_assignment`): the
/// 1-based number of the paper's copy of an author file a stand-in binding answers
/// (`binding::content::read_stand_in_copy`), while that copy is read, while a hook it registered runs, or `0`.
pub fn copy_provides() -> u32 { COPY_PROVIDES.with(|c| c.get()) }

/// Enter copy `id`'s provides-mode (`0`: suspend it, for a file the copy loads), until the matching
/// [`leave_copy_provides`]. A stack, not a guard: the scope markers `\lx@copy@provides@begin{N}` and
/// `\lx@copy@provides@end` wrapped around a hook the copy registers bracket a token run, not a Rust call. Entering
/// from no mode (or leaving to none) puts `\AddToHook`'s registration seam in place (or back out,
/// `state::swap_copy_provides_hook_seam`).
pub fn enter_copy_provides(id: u32) {
  let previous = COPY_PROVIDES.with(|c| c.get());
  COPY_PROVIDES_STACK.with(|s| s.borrow_mut().push(previous));
  crate::state::enter_copy_provides_frame(id, previous);
  if (previous == 0) != (id == 0) {
    crate::state::swap_copy_provides_hook_seam(id != 0);
  }
  // (the copy's code is a file's: locked, as a raw read is, `load_tex_definitions` — also when its
  // `\AtEndOfClass` code runs inside `\documentclass`'s unlocked after-digestion)
  if id != 0 {
    crate::common::local_assignments::local_state_unlocked(false);
  }
  COPY_PROVIDES.with(|c| c.set(id));
}

/// Leave the provides-mode [`enter_copy_provides`] entered: the interrupted one is in force again. An unmatched leave
/// is ignored.
pub fn leave_copy_provides() {
  let Some(previous) = COPY_PROVIDES_STACK.with(|s| s.borrow_mut().pop()) else {
    return;
  };
  let current = COPY_PROVIDES.with(|c| c.replace(previous));
  if current != 0 {
    crate::common::local_assignments::expire_state_unlocked();
  }
  crate::state::leave_copy_provides_frame();
  if (previous == 0) != (current == 0) {
    crate::state::swap_copy_provides_hook_seam(previous != 0);
  }
}

/// RAII form of [`enter_copy_provides`]/[`leave_copy_provides`], for a Rust-bracketed scope (the copy's read, a
/// nested load's suspension).
pub struct CopyProvidesGuard;

impl CopyProvidesGuard {
  pub fn new(id: u32) -> Self {
    enter_copy_provides(id);
    CopyProvidesGuard
  }
}

impl Drop for CopyProvidesGuard {
  fn drop(&mut self) { leave_copy_provides(); }
}

thread_local! {
  static STAND_IN_LOADING: Cell<u32> = const { Cell::new(0) };
  static STAND_IN_OWN_CODE: Cell<bool> = const { Cell::new(false) };
  static STAND_IN_LOADER_OWN_CODE: Cell<bool> = const { Cell::new(false) };
  static SETTER_WATCH: Cell<bool> = const { Cell::new(false) };
}

/// Are frontmatter setter calls observed ([`watch_setter_calls`])? Read at every macro call, so a flag.
pub fn setter_watch() -> bool { SETTER_WATCH.with(|c| c.get()) }

/// Observe the calls of the setters `names` from now on (`state::note_setter_call`): a stand-in's copy stored a
/// default of their field, which the document's own call replaces whatever its value.
pub fn watch_setter_calls(names: &[&str]) {
  crate::state::watch_setters(names);
  SETTER_WATCH.with(|c| c.set(true));
}

/// Is a binding that stands in for an author's file loading ([`StandInLoadGuard`]), itself or a file it loads?
pub fn stand_in_loading() -> bool { STAND_IN_LOADING.with(|c| c.get()) != 0 }

/// Is the code running now a stand-in binding's own ([`StandInLoadGuard::new`]), not a file it loads
/// ([`StandInLoadGuard::suspend`])?
pub fn stand_in_own_code() -> bool { STAND_IN_OWN_CODE.with(|c| c.get()) }

/// Did a stand-in binding's own code request the file now loading ([`StandInLoadGuard::suspend`] keeps the flag it
/// suspended)? A stand-in binding that file is joins the loading binding's command set only then
/// ([`StandInLoadGuard::loaded_by`]).
pub fn stand_in_loader_own_code() -> bool { STAND_IN_LOADER_OWN_CODE.with(|c| c.get()) }

/// RAII guard: while a binding that stands in for an author's file loads (`binding::content::input_definitions`),
/// every name assigned is recorded as that binding's (`state::State::stand_in_meanings`), whatever made it — its
/// `DefMacro!`s, its `Let!`s and `RawTeX`, the bindings it requires — so the paper's copy read after it only provides
/// them (`state::note_copy_provides_assignment`). The names its own code assigns, not a file it loads
/// ([`StandInLoadGuard::suspend`]: OmniBus, natbib, a nested stand-in's own), are also its command set
/// (`state::State::stand_in_last_load`), which the copy must define to be the file the binding stands in for
/// (`binding::content` `copy_is_the_stood_in_file`).
pub struct StandInLoadGuard {
  own_code: bool,
  load:     bool,
  /// (a load: its names join the stand-in binding loading it, whose own code requested it; a suspension: the loader
  /// flag it replaced)
  from_own: bool,
}

impl StandInLoadGuard {
  /// A stand-in binding the code running now requests (`binding::content::input`'s `.tex` route): it joins the command
  /// set of a stand-in binding loading it when that is the binding's own code.
  pub fn new() -> Self { Self::loaded_by(stand_in_own_code()) }

  /// A stand-in binding loads, requested by a stand-in binding's own code when `from_own_code`
  /// (`binding::content::input_definitions`, [`stand_in_loader_own_code`]): its names join that binding's command set
  /// at its end (`state::end_stand_in_load`) — the diagrams binding's diagrams.tex (hep-ph9609379) —, and not when a
  /// file the outer binding loads requested it: OmniBus's `RequirePackage!("aas_macros")` (omnibus_cls.rs:71) under
  /// asme2ej (2309.12017), nature-pre (2102.01782, 2111.09589), cimart (2412.00032).
  pub fn loaded_by(from_own_code: bool) -> Self {
    STAND_IN_LOADING.with(|c| c.set(c.get() + 1));
    crate::state::begin_stand_in_load();
    StandInLoadGuard {
      own_code: STAND_IN_OWN_CODE.with(|c| c.replace(true)),
      load:     true,
      from_own: from_own_code,
    }
  }

  /// A file a stand-in binding loads (`binding::content::input_definitions`): its names are the binding's to own, but
  /// not the binding's command set, until the guard drops.
  pub fn suspend() -> Self {
    let own_code = STAND_IN_OWN_CODE.with(|c| c.replace(false));
    StandInLoadGuard {
      own_code,
      load: false,
      from_own: STAND_IN_LOADER_OWN_CODE.with(|c| c.replace(own_code)),
    }
  }
}

impl Default for StandInLoadGuard {
  fn default() -> Self { Self::new() }
}

impl Drop for StandInLoadGuard {
  fn drop(&mut self) {
    STAND_IN_OWN_CODE.with(|c| c.set(self.own_code));
    if self.load {
      STAND_IN_LOADING.with(|c| c.set(c.get().saturating_sub(1)));
      crate::state::end_stand_in_load(self.from_own);
    } else {
      STAND_IN_LOADER_OWN_CODE.with(|c| c.set(self.from_own));
    }
  }
}

/// Forget every provides-mode: a new conversion starts with none (`reset_thread_engine`).
pub fn reset_copy_provides() {
  STAND_IN_LOADING.with(|c| c.set(0));
  STAND_IN_OWN_CODE.with(|c| c.set(false));
  STAND_IN_LOADER_OWN_CODE.with(|c| c.set(false));
  SETTER_WATCH.with(|c| c.set(false));
  COPY_PROVIDES.with(|c| c.set(0));
  COPY_PROVIDES_STACK.with(|s| s.borrow_mut().clear());
}

/// Run `f` with `origin` as the current definition origin.
pub fn with_origin<R>(origin: DefinitionOrigin, f: impl FnOnce() -> R) -> R {
  let _guard = OriginGuard::new(origin);
  f()
}
