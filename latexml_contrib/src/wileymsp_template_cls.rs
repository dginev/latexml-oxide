//! Stub for WileyMSP-template.cls (Wiley Mathematical Sciences Publishers).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("fancyhdr");
  RequirePackage!("ragged2e");

  DefMacro!(T_CS!("\\begin{affiliations}"), None, "");
  DefMacro!(T_CS!("\\end{affiliations}"), None, "");
  DefMacro!("\\correspondingauthor[]{}", "");
  DefMacro!("\\corres{}", "");
});
