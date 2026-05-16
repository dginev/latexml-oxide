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
  InputDefinitions!("iccv", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
