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

  /// Is this definition a binding's — its own code, a fallback class's guess, a stand-in, a borrowing — rather than the
  /// kernel's (a pool, a dump, a format) or the document's (`state::note_dropped_argument`)?
  pub fn is_binding_made(self) -> bool {
    matches!(
      self,
      DefinitionOrigin::Binding
        | DefinitionOrigin::Fallback
        | DefinitionOrigin::Stub
        | DefinitionOrigin::Borrowed
    )
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
  static STAND_IN_BASE_CLASS: Cell<bool> = const { Cell::new(false) };
  /// Which setter watches are on: [`COPY_DEFAULT_WATCH`], [`FRONTMATTER_WATCH`] (one flag, read at every macro call).
  static SETTER_WATCH: Cell<u8> = const { Cell::new(0) };
}

/// [`watch_setter_calls`]' watch: a stand-in copy stored a default of the setters' fields.
const COPY_DEFAULT_WATCH: u8 = 1;
/// [`watch_frontmatter_setter_calls`]' watch: the frontmatter observer's.
const FRONTMATTER_WATCH: u8 = 2;
/// [`watch_document_calls`]' watch: the title-page check's (`state::note_document_call`).
const DOCUMENT_CALL_WATCH: u8 = 4;
/// [`watch_dropped_arguments`]' watch: the title-page check's (`state::note_dropped_argument`).
const DROPPED_ARGUMENT_WATCH: u8 = 8;
/// [`watch_store_calls`]' watch: the title-page check's (`state::note_store_call`).
const STORE_CALL_WATCH: u8 = 16;

/// Are frontmatter setter calls observed ([`watch_setter_calls`], [`watch_frontmatter_setter_calls`])? Read at every
/// macro call, so a flag.
pub fn setter_watch() -> bool { SETTER_WATCH.with(|c| c.get()) != 0 }

/// What a macro's call must be for one of the watches on to see it, read at every macro call: `None` when no watch is on,
/// `Some(true)` when only [`watch_document_calls`]' and [`watch_store_calls`]' are, which see only a macro with parameters
/// that LaTeXML or a raw class's own file made. Those two stay on from a conversion's start until its title page, or to
/// its end in a document with none, so a package's macro calls in the document's text would otherwise all pay for the
/// watch's cold path.
#[inline(always)]
pub fn call_watch() -> Option<bool> {
  let watches = SETTER_WATCH.with(|c| c.get());
  (watches != 0).then_some(watches & !(DOCUMENT_CALL_WATCH | STORE_CALL_WATCH) == 0)
}

/// Is [`watch_frontmatter_setter_calls`]' watch on?
pub fn frontmatter_watch() -> bool { SETTER_WATCH.with(|c| c.get()) & FRONTMATTER_WATCH != 0 }

/// Is [`watch_setter_calls`]' watch on?
pub fn copy_default_watch() -> bool { SETTER_WATCH.with(|c| c.get()) & COPY_DEFAULT_WATCH != 0 }

/// Is one of the setter watches of [`watch_setter_calls`] and [`watch_frontmatter_setter_calls`] on (the calls
/// `state::note_setter_call` reads)?
pub fn setter_name_watch() -> bool {
  SETTER_WATCH.with(|c| c.get()) & (COPY_DEFAULT_WATCH | FRONTMATTER_WATCH) != 0
}

/// Is [`watch_document_calls`]' watch on?
pub fn document_call_watch() -> bool { SETTER_WATCH.with(|c| c.get()) & DOCUMENT_CALL_WATCH != 0 }

/// Is [`watch_dropped_arguments`]' watch on?
pub fn dropped_argument_watch() -> bool {
  SETTER_WATCH.with(|c| c.get()) & DROPPED_ARGUMENT_WATCH != 0
}

/// Is [`watch_store_calls`]' watch on, the gullet reading the document's text?
pub fn store_call_watch() -> bool { SETTER_WATCH.with(|c| c.get()) & STORE_CALL_WATCH != 0 }

/// Observe (`true`) or stop observing the required arguments a binding's macro drops (`state::note_dropped_argument`):
/// while a frontmatter entry the document gave is digested (latexml_engine `title_page_check`); the watch it replaces.
pub fn watch_dropped_arguments(on: bool) -> bool {
  SETTER_WATCH.with(|c| {
    let was = c.get() & DROPPED_ARGUMENT_WATCH != 0;
    c.set(if on {
      c.get() | DROPPED_ARGUMENT_WATCH
    } else {
      c.get() & !DROPPED_ARGUMENT_WATCH
    });
    was
  })
}

thread_local! {
  /// Whether [`watch_document_calls`] is on, and whether the gullet reads a mouth of the document's text now
  /// ([`note_document_mouth`]): the watch flag is set only while both hold, so no macro a package, a class or a
  /// binding's own text calls takes the watched path.
  static DOCUMENT_CALLS_WATCHED: Cell<(bool, bool)> = const { Cell::new((false, false)) };
  /// Whether [`watch_store_calls`] is on: the watch flag is set only while the gullet reads a mouth of the document's
  /// text too, and it is off from the title page on, so a call in the body pays nothing.
  static STORE_CALLS_WATCHED: Cell<bool> = const { Cell::new(false) };
  /// Whether a raw class or a raw file of the document's own directory is being read ([`note_reading_own_file`]).
  static READING_OWN_FILE: Cell<bool> = const { Cell::new(false) };
}

/// Set the watch flag of [`watch_document_calls`] from `DOCUMENT_CALLS_WATCHED`.
fn sync_document_call_watch() {
  let (on, document_mouth) = DOCUMENT_CALLS_WATCHED.with(Cell::get);
  let stores = STORE_CALLS_WATCHED.with(Cell::get);
  SETTER_WATCH.with(|c| {
    let mut flags = c.get() & !(DOCUMENT_CALL_WATCH | STORE_CALL_WATCH);
    if on && document_mouth {
      flags |= DOCUMENT_CALL_WATCH;
    }
    if stores && document_mouth {
      flags |= STORE_CALL_WATCH;
    }
    c.set(flags);
  });
}

/// Observe (`true`) or stop observing the calls of LaTeXML's own macros the document's text makes, their arguments read
/// from it (`state::note_document_call`): the title-page check's second source, what the document gave a binding's
/// frontmatter commands (latexml_engine `base_utilities::frontmatter::title_page_check`). On from a conversion's
/// start until its frontmatter is sealed; observed only while the gullet reads the document's text
/// ([`note_document_mouth`]).
pub fn watch_document_calls(on: bool) {
  DOCUMENT_CALLS_WATCHED.with(|c| c.set((on, c.get().1)));
  sync_document_call_watch();
}

/// Observe (`true`) or stop observing the calls of the setters a raw class or a raw file of the document's own directory
/// made (`state::note_store_call`): the title-page check's third source (latexml_engine
/// `base_utilities::frontmatter::title_page_check`). On from a conversion's start until the title page; observed only
/// while the gullet reads the document's text ([`note_document_mouth`]).
pub fn watch_store_calls(on: bool) {
  STORE_CALLS_WATCHED.with(|c| c.set(on));
  sync_document_call_watch();
}

/// A raw file starts being read as definitions: is it a class or a file of the document's own directory
/// (`binding::content::load_tex_definitions`)? What it says before the load is read, for its end. What a `\def` makes
/// meanwhile is such a file's ([`crate::definition::expandable::Expandable::made_in_own_file`]): definitions record no
/// source file of their own (their `locator` is a stub, Expandable.pm's is unported).
pub fn note_reading_own_file(own: bool) -> bool { READING_OWN_FILE.with(|c| c.replace(own)) }

/// Is a raw class or a raw file of the document's own directory being read ([`note_reading_own_file`])?
pub fn reading_own_file() -> bool { READING_OWN_FILE.with(Cell::get) }

/// The gullet's current mouth changed: does it read the document's own text (`Mouth::is_document_text`)?
pub fn note_document_mouth(document_text: bool) {
  if DOCUMENT_CALLS_WATCHED.with(Cell::get).1 != document_text {
    DOCUMENT_CALLS_WATCHED.with(|c| c.set((c.get().0, document_text)));
    sync_document_call_watch();
  }
}

/// Observe the calls of the setters `names` from now on (`state::note_setter_call`): a stand-in's copy stored a
/// default of their field, which the document's own call replaces whatever its value.
pub fn watch_setter_calls(names: &[&str]) {
  crate::state::watch_setters(names);
  SETTER_WATCH.with(|c| c.set(c.get() | COPY_DEFAULT_WATCH));
}

/// Observe the expansions of the frontmatter setters `names` for this conversion (`state::note_setter_call`,
/// `state::open_frontmatter_setter`: whose expansion an entry the frontmatter API queues was made in — latexml_engine's
/// observer ledger), or stop observing them (`None`).
pub fn watch_frontmatter_setter_calls(names: Option<&[&str]>) {
  crate::state::watch_frontmatter_setters(names.unwrap_or_default());
  SETTER_WATCH.with(|c| {
    c.set(match names {
      Some(_) => c.get() | FRONTMATTER_WATCH,
      None => c.get() & !FRONTMATTER_WATCH,
    })
  });
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

/// Is the code running now a class a stand-in binding loads ([`StandInLoadGuard::suspend_for`]: its base class, the
/// pool's article, report or book, or the OmniBus guess built on them), not the binding's own code nor a package?
/// The text defaults such a class assigns (`\refname`, `\bibname`, `\abstractname`) are not the binding's
/// (`state::State::stand_in_base_defaults`).
pub fn stand_in_base_class() -> bool { STAND_IN_BASE_CLASS.with(|c| c.get()) }

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
  /// (the base-class flag replaced: [`stand_in_base_class`])
  base:     bool,
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
      base:     STAND_IN_BASE_CLASS.with(|c| c.replace(false)),
    }
  }

  /// A file a stand-in binding loads (`binding::content::input_definitions`): its names are the binding's to own, but
  /// not the binding's command set, until the guard drops.
  pub fn suspend() -> Self { Self::suspend_for(false) }

  /// [`StandInLoadGuard::suspend`] for a file that is a class (`class`, a `.cls`) or not: a class the binding's own
  /// code loads is its base class ([`stand_in_base_class`]), and so is every class that class loads; a package any of
  /// them loads is not (the bindings a binding requires stay its own).
  pub fn suspend_for(class: bool) -> Self {
    let own_code = STAND_IN_OWN_CODE.with(|c| c.replace(false));
    let base = STAND_IN_BASE_CLASS.with(|c| c.replace(class && (own_code || c.get())));
    StandInLoadGuard {
      own_code,
      load: false,
      from_own: STAND_IN_LOADER_OWN_CODE.with(|c| c.replace(own_code)),
      base,
    }
  }
}

impl Default for StandInLoadGuard {
  fn default() -> Self { Self::new() }
}

impl Drop for StandInLoadGuard {
  fn drop(&mut self) {
    STAND_IN_OWN_CODE.with(|c| c.set(self.own_code));
    STAND_IN_BASE_CLASS.with(|c| c.set(self.base));
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
  STAND_IN_BASE_CLASS.with(|c| c.set(false));
  // (the frontmatter observer's watch and the title-page check's are their own, set at their resets:
  // `watch_frontmatter_setter_calls`, `watch_document_calls`, `watch_dropped_arguments`, `watch_store_calls`)
  SETTER_WATCH.with(|c| {
    c.set(
      c.get()
        & (FRONTMATTER_WATCH | DOCUMENT_CALL_WATCH | DROPPED_ARGUMENT_WATCH | STORE_CALL_WATCH),
    )
  });
  COPY_PROVIDES.with(|c| c.set(0));
  COPY_PROVIDES_STACK.with(|s| s.borrow_mut().clear());
}

/// Run `f` with `origin` as the current definition origin.
pub fn with_origin<R>(origin: DefinitionOrigin, f: impl FnOnce() -> R) -> R {
  let _guard = OriginGuard::new(origin);
  f()
}
