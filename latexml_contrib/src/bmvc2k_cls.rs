//! Stub for bmvc2k.cls (BMVC British Machine Vision Conference).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");

  // bmvc2k frontmatter (L167+).
  DefMacro!("\\bmvaOneDot", "");
  DefMacro!("\\bmvaHangBox{}", "#1");
  DefMacro!("\\addauthor{}{}{}", "");
  DefMacro!("\\addinstitution{}", "");
});
