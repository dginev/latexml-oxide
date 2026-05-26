//! refstyle.sty — flexible cross-reference styling (`\newref{type}{...}`,
//! `\vref`, `\Vref`, `\Ref`, ...).
//!
//! refstyle.sty's refstyle.cfg has `\newref{eq}{...}` which tries
//! to redefine `\eqref` after `\RS@removedef{eqref}`. The
//! `\RS@removedef` does `\let\eqref\@undefined` — in TeX this
//! makes `\eqref` effectively undefined, but our `\@ifundefined`
//! test (`\ifx\csname...\endcsname\relax`) doesn't recognize
//! `\@undefined` as `\relax`, so the test sees `\eqref` as still
//! defined and refstyle's `\RS@notdefinable` fires
//! `\PackageError{refstyle}{Command \eqref already defined}`.
//!
//! Witnesses: arXiv:2009.10518, arXiv:1804.06350 — both load
//! refstyle (and additionally `cleveref` which provides
//! `\eqref` anyway). Perl LaTeXML has no refstyle binding
//! (INCLUDE_STYLES=false skips), zero errors.
//!
//! Stub: no-op binding. The user-facing `\newref` / `\vref` /
//! `\Vref` / `\Ref` are stubbed to passthrough `\ref`.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "refstyle.sty",
    "refstyle.sty is minimally stubbed — \\newref/\\Vref/\\vref pass through to \\ref."
  );
  // refstyle's main public API. We don't honor the per-type
  // templating; just dispatch to plain \ref so the labels still
  // render correctly (just without the type-prefix prose).
  DefMacro!("\\newref [] {} []", "");
  DefMacro!("\\Vref {}", "\\ref{#1}");
  DefMacro!("\\vref {}", "\\ref{#1}");
  DefMacro!("\\Ref {}", "\\ref{#1}");
  DefMacro!("\\refstyle {}", "");
  DefMacro!("\\rangeref {} {}", "\\ref{#1}~--~\\ref{#2}");
  DefMacro!("\\Rangeref {} {}", "\\ref{#1}~--~\\ref{#2}");
  DefMacro!("\\nrefrange [] {} {}", "\\ref{#2}~--~\\ref{#3}");
  DefMacro!("\\extdef {} {}", "");
  // \refstylefirst — toggles "use long-form name on first ref"; no-op.
  def_macro_noop("\\refstylefirst")?;
  // RSfooform — formatter templates; ignored.
});
