//! Stub for chemformula.sty (chemical formulas).
//!
//! Maps \ch{...} to mhchem's \ce since both render similarly for our
//! HTML/XML output where the chemistry notation isn't fully styled.
use latexml_package::prelude::*;

LoadDefinitions!({
  // Pass `version=4`: mhchem now raw-loads the real package (2026-06-27), which
  // emits "You did not specify a 'version' option" if none is given. The old
  // mhchem stub accepted a bare load silently; the real package does not.
  RequirePackage!("mhchem", options => vec!["version=4".to_string()]);
  // chemformula 4.x is an expl3 LaTeX3 package; the INCLUDE_STYLES
  // post-binding raw load calls \ProcessKeysPackageOptions at
  // chemformula.sty L481. Without l3keys2e + xparse loaded first,
  // the raw load errors with \ProcessKeysPackageOptions undefined.
  // Driver cluster: stage11_v3 2504.13749 (chemformula raw-load).
  RequirePackage!("l3keys2e");
  RequirePackage!("xparse");
  // Mirror chemformula.sty L29 `\RequirePackage{tikz,amsmath,xfrac,nicefrac}`
  // faithfully — Perl has no chemformula binding, so it raw-loads the real
  // chemformula.sty and pulls in ALL of these transitively. `\sfrac` (xfrac)
  // becomes available to the document (witness 2006.07679: `\sfrac{\theta}{2}`
  // in plain math, 1 error → 0). `tikz` was previously omitted "to keep the
  // stub light" (the stub renders `\ch` via mhchem `\ce`, not chemformula's
  // tikz-drawn arrows), but that omission is a DIVERGENCE: the real chemformula
  // requires tikz, and tikz → pgf → pgfsys-latexml loads `xcolor`. A paper that
  // does `\PassOptionsToPackage{table}{xcolor}` then relies on chemformula to
  // pull in xcolor (with the table option → `\rowcolors`/colortbl) saw
  // `\rowcolors` undefined where Perl had it. Witness 1809.04023 (revtex4-1 +
  // `\PassOptionsToPackage{table}{xcolor}` + chemformula + `\rowcolors`):
  // 1 error → 0. So require tikz too, matching chemformula.sty L29 exactly.
  RequirePackage!("tikz");
  RequirePackage!("xfrac");
  RequirePackage!("nicefrac");
  Let!("\\ch", "\\ce");
  Let!("\\chcpd", "\\ce");
  def_macro_noop("\\chsetup{}")?;
  def_macro_noop("\\setchemformula{}")?;
});
