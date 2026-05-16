//! Stub for spie.cls (SPIE conference proceedings).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // spie.cls L107: \authorinfo{...} for author footnote.
  DefMacro!("\\authorinfo{}", "");
  DefMacro!("\\skiplinehalf", "");
  DefMacro!("\\supit{}", "\\textsuperscript{#1}");
});
