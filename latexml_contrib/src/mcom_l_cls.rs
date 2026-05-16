//! Stub for mcom-l.cls / proc-l.cls / tran-l.cls (AMS journal classes).
use latexml_package::prelude::*;

LoadDefinitions!({
  // mcom-l L30: \LoadClass{amsart}
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("url");

  // AMS journal frontmatter.
  DefMacro!("\\subjclass[]{}", "");
  DefMacro!("\\keywords{}", "");
  DefMacro!("\\thanks{}", "");
  DefMacro!("\\address{}", "");
  DefMacro!("\\curraddr{}", "");
  DefMacro!("\\email{}", "");
  DefMacro!("\\urladdr{}", "");
  DefMacro!("\\dedicatory{}", "");
});
