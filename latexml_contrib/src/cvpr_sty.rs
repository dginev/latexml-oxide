//! Stub for cvpr.sty / iccv.sty / iccvw.sty (computer vision conference style).
//!
//! cvpr.sty redefines \title to save the argument in \thetitle so it can
//! be reused (typically by \maketitlesupplementary). Our raw load of
//! cvpr.sty appears not to wire this up reliably; bind cvpr defensively
//! to define \thetitle as a no-op default, plus stub the rebuttal/
//! supplementary frontmatter.
use latexml_package::prelude::*;

LoadDefinitions!({
  // \thetitle: default-empty, gets overridden when user calls \title{...}.
  DefMacro!("\\thetitle", "");
  DefMacro!("\\maketitlesupplementary", "");

  // cvpr.sty supplies these toggles via etoolbox — provide as fallback.
  DefConditional!("\\ifcvprfinal");
  DefConditional!("\\ifcvprrebuttal");
  DefConditional!("\\ifcvprpagenumbers");

  InputDefinitions!("cvpr", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
