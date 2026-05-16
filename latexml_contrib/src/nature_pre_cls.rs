//! Stub for nature-pre.cls (Nature pre-print template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Nature pre L67 \newenvironment{affiliations} — list of author
  // affiliations. Treat as transparent (no semantic XML wrapper).
  DefMacro!(T_CS!("\\begin{affiliations}"), None, "");
  DefMacro!(T_CS!("\\end{affiliations}"), None, "");
  DefMacro!("\\correspondingauthor[]{}", "");
  DefMacro!("\\thanks{}", "");
});
