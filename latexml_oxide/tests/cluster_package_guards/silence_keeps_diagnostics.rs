//! `silence.sty` must never cost us a real diagnostic
//! (`latexml_contrib/src/silence_sty.rs`).
//!
//! Unlike the `arxiv.sty` sibling, the silence binding is deliberately NOT
//! gated on `INCLUDE_STYLES`: it pre-empts the raw `.sty` in every
//! configuration. The reason is measurable. The real silence.sty rebinds
//! `\PackageError` / `\ClassError` / `\@latex@error` / `\GenericError`
//! (silence.sty L582-599) so that `\ErrorsOff` drops messages before they
//! are printed — and under LaTeXML those are the very definitions that turn
//! a package's error into an `Error:` line. Measured on the fixture below,
//! same-host Perl 0.8.8 with `--includestyles` reports **0 errors**; without
//! `\usepackage{silence}` the same document reports **1**. The raw load
//! silently downgrades a genuine diagnostic.
//!
//! The binding models only what silence contributes to the *document*
//! (nothing) and leaves the error/warning definitions alone, so the
//! diagnostic survives. This test pins that: the run must still report the
//! `boompkg` error even with silence loaded and `\ErrorsOff` in force.

const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{silence}\n\
    \\ErrorsOff\n\
    \\usepackage{boompkg}\n\
    \\begin{document}\n\
    x\n\
    \\end{document}\n";

const STY: &str = "\\ProvidesPackage{boompkg}\n\
    \\PackageError{boompkg}{Deliberate boom}{}\n";

#[test]
fn silence_errorsoff_does_not_swallow_a_package_error() {
  let (stderr, _xml) =
    super::convert_files_with(TEX, &[("boompkg.sty", STY)], Some("[rawstyles]latexml.sty"));
  assert!(
    stderr.contains("Deliberate boom"),
    "silence + \\ErrorsOff must not suppress the boompkg error:\n{stderr}",
  );
}
