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
use std::cell::Cell;

/// Who made a definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionOrigin {
  /// Made before any loader seam set an origin (engine bootstrap) — ours.
  Unknown,
  /// Restored from the plain format dump (`<embedded:plain>`).
  Plain,
  /// Restored from the LaTeX format dump (`<embedded:latex>`).
  LatexDump,
  /// A Rust pool (`InnerPool!`/`LoadPool!`: TeX.pool, LaTeX.pool, constructs).
  Pool,
  /// A compiled binding's own code (`DefMacro!`/`RawTeX!` in a `*_sty.rs`).
  Binding,
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

/// Run `f` with `origin` as the current definition origin.
pub fn with_origin<R>(origin: DefinitionOrigin, f: impl FnOnce() -> R) -> R {
  let _guard = OriginGuard::new(origin);
  f()
}
