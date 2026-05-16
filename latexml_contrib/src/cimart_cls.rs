//! Stub for `cimart` class (CiM = Communications in Mathematics).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // CiM frontmatter — gobble.
  DefMacro!("\\YEAR{}", "");
  DefMacro!("\\VOLUME{}", "");
  DefMacro!("\\ISSUE{}", "");
  DefMacro!("\\NUMBER{}", "");
  DefMacro!("\\DOI{}", "");
  DefMacro!("\\msc{}", "");
  DefMacro!("\\authorinfo{}", "");
  DefMacro!("\\EditInfo{}{}{}", "");
});
