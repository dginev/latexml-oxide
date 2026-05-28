//! No-op stub for constants.sty.
//!
//! constants.sty (TL — "Labeling and numbering constants") provides
//! `\newconstantfamily`, `\C`, `\Cl`, `\refconstant`, `\resetconstant`.
//! Its end-of-document bookkeeping (constants.sty L143-148) does a *raw*
//! `\if@filesw … \makeatletter \input\jobname.aux \fi` to re-read the
//! label `.aux` for the "constants may have changed, rerun" check.
//!
//! Our engine never opens `\@mainaux` at *runtime* (the kernel's
//! `\openout\@mainaux\jobname.aux` is baked into the dump at build
//! time, with the build-time jobname), so no in-memory `<jobname>.aux`
//! buffer exists; the raw `\input\jobname.aux` then fails with
//! `Error:missing_file:<jobname>.aux`. (Unlike LaTeX's silent `\@input`,
//! constants.sty uses the hard `\input`.)
//!
//! Perl LaTeXML never observes this: its default TEXINPUTS excludes
//! `/usr/share/texlive`, so constants.sty is reported missing-file and
//! skipped — providing ZERO constants functionality. Verified on
//! arXiv:2002.05335: Perl emits `Warning:missing_file:constants` and
//! completes (its residual errors are unrelated math-mode issues).
//!
//! Match Perl's effective behavior with a no-op stub (RequirePackage
//! the real `keyval` dep; no-op the user-facing constants API). All 70
//! R-stage papers blocked on `<jobname>.aux` were checked: every one is
//! CONVERR_1 (the single `.aux` error) and NONE invoke `\C`/`\Cl`/
//! `\newconstantfamily`, so the stub costs no document content.
//!
//! Witnesses (CONVERR_1 → OK): 2004.04403, 2002.05335, 1501.05396,
//! 1503.01423, 1505.03199, 1506.02544, 1506.03382, 1506.03705,
//! 1508.03002, 1509.02505, 1512.06906, 1601.01096, 1603.05947,
//! 1606.09238, … (70 total).

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("keyval");
  // User-facing constants API — no-op (matches Perl's missing-file
  // skip; no observed paper uses these).
  def_macro_noop("\\newconstantfamily{}{}")?;
  def_macro_noop("\\renewconstantfamily{}{}")?;
  def_macro_noop("\\resetconstant[]")?;
  def_macro_noop("\\C[]")?;
  def_macro_noop("\\Cl[]{}")?;
  def_macro_noop("\\refconstant{}")?;
});
