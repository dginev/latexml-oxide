//! backref.sty — back-references from bibliography to citations
//! (part of the Hyperref Bundle).
//!
//! backref adds a "Cited on pages X, Y" link at the end of each
//! bibliography entry, pointing back to the citation locations.
//! It does this by redefining `\@bibitem` / `\@lbibitem` to wrap
//! the bibitem body with a `\BR@bibitem` hook.
//!
//! Raw-loading backref.sty into our engine is fragile: backref's
//! redefinitions of `\@bibitem`/`\@lbibitem` chain with paper-
//! bundled bibliography helpers (e.g. myharvard's `\harvarditem`
//! which writes to `.aux` then calls `\item[]`) — the resulting
//! expansion loops indefinitely (witness 1107.0498: amsart +
//! `\usepackage[hyperpageref]{backref}` + paper-local myharvard.sty
//! → Convert TIMEOUT in `.bbl` processing; Perl handles the same
//! input with the same 4 undefined-macro errors but completes).
//!
//! Perl LaTeXML has no backref binding either; with default
//! `INCLUDE_STYLES=false`, raw backref.sty is not loaded — Perl
//! emits "missing binding" and continues without back-references.
//!
//! Match Perl: stub the user-facing API as no-ops. We lose the
//! back-reference links, but the bibliography itself renders
//! correctly.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "backref.sty",
    "backref.sty is minimally stubbed — back-references from bibliography to citations are not rendered."
  );
  // backref's user-facing API: setup, format, print.
  DefMacro!("\\backrefsetup{}", "");
  DefMacro!("\\backrefprint", "");
  DefMacro!("\\backrefparscanfalse", "");
  DefMacro!("\\backrefparscantrue", "");
  // Language definitions (no-op; we don't render the back-ref
  // section). These are the language hooks backref normally fills.
  for cs in &[
    "\\backrefenglish",
    "\\backrefgerman",
    "\\backreffrench",
    "\\backrefspanish",
    "\\backrefbrazil",
    "\\backrefafrikaans",
    "\\backrefitalian",
  ] {
    def_macro_noop(cs)?;
  }
});
