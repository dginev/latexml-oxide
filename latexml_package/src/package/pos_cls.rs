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

  DefMacro!("\\ShortTitle{}",     "\\lx@add@toctitle{#1}");
  DefMacro!("\\PoScopydate{}",    "\\lx@add@date[role=accepted]{#1}");
  DefMacro!("\\PACSname",      "\\textbf{PACS}");
  DefMacro!("\\PACS{}",           "\\lx@add@classification[scheme=pacs,name={\\PACSname:~}]{#1}");
  // \FullConference{name} — conference identification text. Round-34
  // surpass-Perl: preserve as ltx:note frontmatter.
  DefMacro!("\\FullConference{}",
    "\\@add@frontmatter{ltx:note}[role=conference]{#1}");

  DefMacro!("\\Jmath", "J");
});
