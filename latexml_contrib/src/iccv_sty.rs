//! Stub for iccv.sty / iccvw.sty (ICCV conference style).
//!
//! Same \thetitle pattern as cvpr.sty.
use latexml_package::prelude::*;

LoadDefinitions!({
  DefMacro!("\\thetitle", "");
  DefMacro!("\\maketitlesupplementary", "");
  DefConditional!("\\ificcvfinal");
  DefConditional!("\\ificcvrebuttal");
  DefConditional!("\\ificcvpagenumbers");
  // \iccvfinalcopy / \iccvPaperID — page-numbering toggles in ICCV
  // templates. Affect print layout only; HTML rendering is invariant.
  // Witness 2 stage-2 papers.
  DefMacro!("\\iccvfinalcopy", "");
  DefMacro!("\\iccvPaperID{}", "");
  InputDefinitions!("iccv", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
