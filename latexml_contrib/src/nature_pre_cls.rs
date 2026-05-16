//! Stub for nature-pre.cls (Nature pre-print template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Nature pre L67 \newenvironment{affiliations} — list of author
  // affiliations. Render body so the affiliation text reaches XML.
  DefMacro!(T_CS!("\\begin{affiliations}"), None, "");
  DefMacro!(T_CS!("\\end{affiliations}"), None, "");
  // Preserve author content.
  DefMacro!("\\correspondingauthor[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}");
  DefMacro!("\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}");
});
