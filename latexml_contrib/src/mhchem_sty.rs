//! mhchem.sty — chemical formula typesetting.
//!
//! TODO(strict-perl-parity): once `latexml_engine` can faithfully
//! handle the expl3 / xparse / chemgreek raw-load chain (currently
//! the gaps are around `\group_begin:` non-boxing-frame handling
//! and l3regex/l3tl-analysis register access during the chemgreek
//! load triggered by the first `\ce{...}`), DELETE this binding so
//! that `\usepackage{mhchem}` raw-loads the actual TL `mhchem.sty`,
//! matching Perl LaTeXML's behavior (Perl has no `mhchem.sty.ltxml`).
//! Driver paper: arXiv:1806.06448 (3 errors → 0 errors with this
//! stub; full chemistry rendering needs the engine fix).
//!
//! Perl LaTeXML has no `mhchem.sty.ltxml` and raw-loads the actual
//! TL `mhchem.sty` (which `\RequirePackage{chemgreek}` →
//! `\RequirePackage{xparse}` → heavy expl3 machinery). Perl's expl3
//! emulation is mature enough that this works.
//!
//! Rust's expl3 emulation has gaps (e.g. `\group_begin:` non-boxing
//! frame handling, `\l__tl_analysis_*_int` register access in
//! l3regex/l3tl-analysis), so the chemgreek raw-load triggered by the
//! first `\ce{...}` invocation leaves the gullet in an unbalanced state
//! (open `\iffalse`, unmatched `{` at end-of-input).
//!
//! Until the expl3 cluster is fixed, this binding intercepts the
//! mhchem load and provides a minimal stub: `\ce{...}` typesets its
//! argument as roman text, no chemistry layout. This is a documented
//! divergence from Perl LaTeXML — the full chemistry rendering needs
//! a real port. Driver paper: 1806.06448 (3 errors → 0 errors).
//!
//! Stubs cover the public mhchem v3/v4 surface most papers actually
//! use: `\ce`, `\cee`, `\cf`, plus `\mhchemoptions` (no-op).

use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Accept both v3 and v4: the package option is `version=N` — handled
  // at \usepackage time but irrelevant to our stub.
  DefMacro!("\\mhchemoptions RequiredKeyVals", "");

  // \ce{<formula>} — chemistry mode. Real mhchem renders subscripts,
  // charges, arrows, etc. Papers invoke \ce{H_2O} / \ce{N_2} both in
  // math context (equation*) AND in text context (paragraphs).
  // \ensuremath wraps body in math mode if not already in math, so
  // `_`/`^` parse as scripts in both contexts. Loses roman-text
  // rendering of plain text chemistry, but avoids cascading errors.
  // Witness: 0907.1390 (`\ce{N_2}, \ce{O_2}` in text → R=3→0).
  DefMacro!("\\ce{}",  "\\ensuremath{#1}");
  DefMacro!("\\cee{}", "\\ensuremath{#1}");
  DefMacro!("\\cf{}",  "\\ensuremath{#1}");

  // \arrow / \chemarrow — used inside \ce arguments. Stub as small text
  // arrow so a `\ce{A \arrow B}` doesn't error if it leaks out.
  DefMacro!("\\chemarrow", "\\rightarrow");
});
