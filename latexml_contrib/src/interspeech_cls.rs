//! Stub for Interspeech.cls (Interspeech conference template).
//!
//! User-bundled conference template; not in TeX Live. The two macros
//! commonly tripped: \interspeechcameraready (camera-ready toggle)
//! and \interspeech (logo/title helpers). Gobble cleanly so the
//! frontmatter doesn't fail.
//! Witness 2409.08589, 2409.08711.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  RequirePackage!("booktabs");

  // Interspeech frontmatter — gobble cleanly.
  DefMacro!("\\interspeechcameraready", "");
  DefMacro!("\\name{}", "");
  DefMacro!("\\address{}", "");
  DefMacro!("\\email{}", "");
  DefMacro!("\\thanks{}", "");
  DefMacro!("\\keywords{}", "");
  DefMacro!("\\copyrightnotice{}", "");
});
