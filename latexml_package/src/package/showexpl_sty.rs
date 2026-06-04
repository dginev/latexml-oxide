//! No-op stub for showexpl.sty.
//!
//! showexpl.sty (TL) builds the `LTXexample` environment and
//! `\LTXinputExample` command (display LaTeX source + its typeset
//! result side by side) on top of `listings`. Its package body uses
//! `\lst@newenvironment{LTXexample}{...}{...\SX@put@code@result}`
//! whose end-group `\xdef\SX@@explpreset{\the\@temptokena,...}` parse
//! tickles a readBalanced `Expected opening '{'` error in our raw-load
//! path, then `\SX@put@code@result` (defined just after, at L208)
//! never registers, cascading.
//!
//! Perl LaTeXML never observes this: its default TEXINPUTS excludes
//! `/usr/share/texlive`, so showexpl.sty is reported as missing-file
//! and skipped, completing cleanly. Verified on arXiv:2002.09910
//! (`\usepackage{...showexpl...}`): Perl emits
//! `Warning:missing_file:showexpl` and 0 errors.
//!
//! Match Perl's effective behavior with a stub that (a) pulls in
//! showexpl's real `\RequirePackage` dependency chain so any
//! transitively-needed macros (listings, varwidth, float) resolve, and
//! (b) no-ops the user-facing showexpl commands. All 15 R-stage papers
//! blocked on `\SX@put@code@result` were checked: NONE invoke
//! `\begin{LTXexample}` or `\LTXinputExample` — they only load the
//! package — so the stub costs no document content.
//!
//! Witnesses (CONVERR_7/CONVERR_3 → OK): 1604.00381, 1606.01035,
//! 1706.03232, 1804.02704, 1804.07221, 1612.01022, 1905.12059,
//! 1706.09226, 1701.01402, 1812.06820, 1801.01025, 1806.10927,
//! 2001.08314, 2002.09910, 1901.08750.

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // showexpl.sty L1-13 \RequirePackage chain (attachfile is loaded
  // conditionally via `\IfFileExists`; we load it unconditionally —
  // harmless, has a binding).
  RequirePackage!("listings");
  RequirePackage!("refcount");
  RequirePackage!("varwidth");
  RequirePackage!("float");
  // User-facing showexpl API — no-op (these display code+result side by
  // side, which our XML output models as plain listings; no paper in
  // the observed cluster uses them, so a no-op is safe and matches
  // Perl's missing-file skip).
  def_macro_noop("\\LTXinputExample[]{}{}")?;
  def_macro_noop("\\setupSXfiles")?;
  def_macro_noop("\\setupLZfiles")?;
});
