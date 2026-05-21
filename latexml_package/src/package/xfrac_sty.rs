use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: xfrac.sty.ltxml. Mirror the real `\RequirePackage` list in
  // /usr/share/texlive/.../l3packages/xfrac/xfrac.sty so transitive
  // CSes from l3keys2e (\ProcessKeysPackageOptions etc.) are
  // available to downstream `\ProvidesExplPackage` consumers like
  // chemformula. Without l3keys2e + xparse, chemformula raw-load
  // hits `\ProcessKeysPackageOptions undefined` at its load-time
  // option-processing call (witness: arXiv:2506.13488 — 16 papers
  // in Stage-13 v3 share this cascade).
  RequirePackage!("amstext");
  RequirePackage!("graphicx");
  RequirePackage!("l3keys2e");
  RequirePackage!("textcomp");
  RequirePackage!("xparse");
  RequirePackage!("nicefrac");

  DefMacro!("\\sfrac[]{} []{}", "\\ensuremath{\\@UnitsNiceFrac{#2}{#4}}");

  DefMacro!("\\DeclareInstance{}{}{}{}", None);
  DefMacro!("\\DeclareCollectionInstance{}{}{}{}{}", None);
  DefMacro!("\\UseCollection{}{}", None);
});
