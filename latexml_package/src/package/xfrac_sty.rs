use std::borrow::Cow;

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

  // The real xfrac.sty declares its `xfrac` object type, the `text`/`math`
  // templates and their default instances through the kernel's xtemplate
  // (latex.ltx:8671 `\DeclareInstance`). The binding used to stub
  // `\DeclareInstance` as a four-argument no-op — GLOBALLY, so once xfrac was
  // loaded (directly, or through chemmacros/chemformula) every later
  // `\DeclareInstance` of any package vanished and its `\UseInstance` raised
  // "The instance 'X' of type 'T' is unknown" (substances-index ×50, tasks'
  // `alphabetize` in schulmathematik; sweep 63). Batch 56ay: raw-load.
  InputDefinitions!("xfrac", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // `\sfrac` stays the semantic fraction (nicefrac's `\@UnitsNiceFrac` →
  // a math fraction) rather than xfrac's scaled/raised text boxes; defined
  // AFTER the raw load so it overrides xfrac's `\NewDocumentCommand`.
  DefMacro!("\\sfrac[]{} []{}", "\\ensuremath{\\@UnitsNiceFrac{#2}{#4}}");
});
