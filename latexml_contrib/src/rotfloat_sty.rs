//! rotfloat.sty — combine `rotating` + `float` for `sidewaystable`/
//! `sidewaysfigure` in user-defined float environments.
//!
//! rotfloat's body is wrapped in `\ifx\@float@HH\undefined\else ...
//! \fi`. The `\else` branch (taken when `\@float@HH` IS defined —
//! which is the case in any document that loaded `float`) contains
//! nested `\if@flstyle\else\end@rotfloat\fi` conditionals. Our
//! conditional tracker doesn't recognize `\@flstyle`'s if-test
//! tokens (defined by the `float` package's `\newif`), so on raw
//! load we report two `\fi`-without-matching-`\if` errors at lines
//! 107 and 120.
//!
//! Perl LaTeXML has no rotfloat binding and its
//! `INCLUDE_STYLES=false` default skips the raw .sty entirely,
//! producing zero errors. Witness arXiv:2101.12526 and
//! arXiv:1804.05845 — both load rotfloat, both get 2 raw-load
//! errors in Rust where Perl emits only one missing-binding
//! warning.
//!
//! Stub the public API as no-ops. `rotating` already provides
//! the `sidewaystable`/`sidewaysfigure` environments that papers
//! actually use; rotfloat's role is just glue between
//! rotating+float, which we don't implement either way.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "rotfloat.sty",
    "rotfloat.sty is minimally stubbed — rotating+float glue is a no-op; sidewaystable/sidewaysfigure still come from rotating."
  );
  // rotfloat.sty L24-25 does `\RequirePackage{float}` then
  // `\RequirePackageWithOptions{rotating}`. Mirror both: float provides
  // `\restylefloat`/`\newfloat`/`\floatstyle` (which papers call
  // directly, e.g. `\restylefloat{figure}` — undefined without it,
  // witness 1604.07054/1808.04014), and rotating provides the
  // sidewaystable/sidewaysfigure environments. float comes first to
  // match the real load order.
  RequirePackage!("float");
  RequirePackage!("rotating");
});
