//! Stub for aomart.cls (Annals of Mathematics).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("fancyhdr");

  // Author metadata (aomart.cls L222+).
  DefMacro!("\\givenname{}", "");
  DefMacro!("\\surname{}", "");
  DefMacro!("\\subject{}{}{}", "");
  DefMacro!("\\published{}", "");
  DefMacro!("\\publishedonline{}", "");
  DefMacro!("\\publicationyear{}", "");
  DefMacro!("\\volumenumber{}", "");
  DefMacro!("\\issuenumber{}", "");
  DefMacro!("\\papernumber{}", "");
  DefMacro!("\\startpage{}", "");
  DefMacro!("\\endpage{}", "");
  DefMacro!("\\doinumber{}", "");
  DefMacro!("\\mrnumber{}", "");
  DefMacro!("\\zblnumber{}", "");
  DefMacro!("\\arxivnumber{}", "");
  DefMacro!("\\version{}", "");
  DefMacro!("\\copyrightnote{}", "");
  DefMacro!("\\formatdate{}", "");
});
