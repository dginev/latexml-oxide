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

  // Interspeech frontmatter — preserve author content.
  DefMacro!("\\interspeechcameraready", "");
  // \name carries the author name in Interspeech templates.
  DefMacro!("\\name{}", "\\author{#1}");
  DefMacro!("\\address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}");
  DefMacro!("\\email{}",
    "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  DefMacro!("\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}");
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\copyrightnotice{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}");
});
