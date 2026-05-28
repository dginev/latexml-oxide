//! Stub for chemformula.sty (chemical formulas).
//!
//! Maps \ch{...} to mhchem's \ce since both render similarly for our
//! HTML/XML output where the chemistry notation isn't fully styled.
use latexml_package::prelude::*;


LoadDefinitions!({
  RequirePackage!("mhchem");
  // chemformula 4.x is an expl3 LaTeX3 package; the INCLUDE_STYLES
  // post-binding raw load calls \ProcessKeysPackageOptions at
  // chemformula.sty L481. Without l3keys2e + xparse loaded first,
  // the raw load errors with \ProcessKeysPackageOptions undefined.
  // Driver cluster: stage11_v3 2504.13749 (chemformula raw-load).
  RequirePackage!("l3keys2e");
  RequirePackage!("xparse");
  // Mirror chemformula.sty L29 `\RequirePackage{tikz,amsmath,xfrac,nicefrac}`:
  // loading chemformula makes `\sfrac` (from xfrac) available to the document.
  // Perl has no chemformula binding — it raw-loads chemformula.sty and pulls
  // in xfrac → `\sfrac` the same way. The stub previously omitted these, so a
  // paper that loads chemformula and then uses `\sfrac` in plain math (NOT
  // inside `\ch`) saw `\sfrac` undefined where Perl had it. Witness 2006.07679
  // (loads chemformula, no `\ch`; uses `\sfrac{\theta}{2}`): 1 error → 0.
  // tikz is intentionally NOT required: the stub renders `\ch` via mhchem
  // `\ce`, not chemformula's tikz-drawn arrows, so tikz is unused — keep the
  // stub light. xfrac transitively brings nicefrac.
  RequirePackage!("xfrac");
  RequirePackage!("nicefrac");
  Let!("\\ch", "\\ce");
  Let!("\\chcpd", "\\ce");
  def_macro_noop("\\chsetup{}")?;
  def_macro_noop("\\setchemformula{}")?;
});
