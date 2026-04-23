use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: PoS.cls.ltxml — Proceedings of Science style.
  // LoadClass("JHEP", withoptions => 1);
  load_class_with_options("JHEP", Tokens!())?;
  RequirePackage!("ifpdf");
  RequirePackage!("times");
  RequirePackage!("mathptmx");
  RequirePackage!("graphicx");

  DefMacro!("\\ShortTitle{}",  "\\@add@frontmatter{ltx:toctitle}{#1}");
  DefMacro!("\\PoScopydate{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");
  DefMacro!("\\PACSname",      "\\textbf{PACS}");
  DefMacro!("\\PACS{}", "\\@add@frontmatter{ltx:classification}[scheme=pacs,name={\\PACSname}]{#1}");
  DefMacro!("\\FullConference{}", "");    // Where to put this?

  DefMacro!("\\Jmath", "J");
});
