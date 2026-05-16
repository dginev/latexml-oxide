//! Stub for asme2ej.cls (ASME journal class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // ASME-specific frontmatter — preserve author content.
  DefMacro!("\\setauthorname{}",
    "\\@add@frontmatter{ltx:note}[role=authorname]{#1}");
  DefMacro!("\\manuscriptnotenumber{}",
    "\\@add@frontmatter{ltx:note}[role=manuscriptno]{#1}");
  DefMacro!("\\confname{}",
    "\\@add@frontmatter{ltx:note}[role=conference]{#1}");
  DefMacro!("\\confyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
});
