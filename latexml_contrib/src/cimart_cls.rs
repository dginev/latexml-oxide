//! Stub for `cimart` class (CiM = Communications in Mathematics).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // CiM frontmatter — preserve author content.
  DefMacro!("\\YEAR{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\VOLUME{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\ISSUE{}",
    "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!("\\NUMBER{}",
    "\\@add@frontmatter{ltx:note}[role=number]{#1}");
  DefMacro!("\\DOI{}",
    "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\msc{}",
    "\\@add@frontmatter{ltx:classification}[scheme=MSC]{#1}");
  DefMacro!("\\authorinfo{}",
    "\\@add@frontmatter{ltx:note}[role=authorinfo]{#1}");
  DefMacro!("\\EditInfo{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=edit-info]{#1 #2 #3}");
});
