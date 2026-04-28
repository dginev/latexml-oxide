//! BibTeX.pool.ltxml — bibliographic-entry processing for LaTeXML.
//!
//! Perl: `LaTeXML/blib/lib/LaTeXML/Engine/BibTeX.pool.ltxml`. Loaded
//! via `LoadPool('BibTeX')` (e.g. from `amsrefs.sty.ltxml`) or as a
//! preload when the conversion mode is BibTeX
//! (`Common/Config.pm:406`: `unshift(... 'BibTeX.pool')`).
//!
//! Status: **skeleton only**. The Perl pool is 956 lines of
//! BibTeX-specific definitions (`\bib`, entry-type constructors,
//! field handlers, BibKey normalization, etc.). They are not yet
//! ported to Rust. This binding exists so every Perl pool has a
//! 1:1 Rust counterpart per the
//! [pool parity audit](../../../docs/POOL_PARITY_AUDIT.md), and it
//! mirrors Perl `BibTeX.pool.ltxml` L19 (`LoadPool('LaTeX')`) so
//! that loading `BibTeX` correctly chains in the LaTeX format.
//!
//! TODO: port BibTeX-specific definitions from
//! `BibTeX.pool.ltxml`. Approximate scope:
//! - `\bib` / `\bibitem` family (Perl ~L80-200)
//! - Bib entry-type constructors (`@article`, `@book`, ... ~L220-500)
//! - Field handlers and helper functions (`CleanBibKey`,
//!   `NormalizeBibKey`, `ProcessBibTeXEntry` — currently stubbed
//!   in `latexml_package::package::amsrefs_sty`).
//! - BibTeX special-character handling (~L800-956).

use crate::prelude::*;

LoadDefinitions!({
  // Perl BibTeX.pool.ltxml L19: `LoadPool('LaTeX')` — BibTeX
  // pool is built on top of the full LaTeX format, since bib
  // entries digest LaTeX-flavored markup in titles/authors/etc.
  LoadPool!("LaTeX");

  // TODO: port the remaining 936+ lines of BibTeX entry-type
  // constructors, field handlers, key normalization, and
  // special-character handling from `BibTeX.pool.ltxml`
  // L20-955. See module docstring above for sub-areas.
});
