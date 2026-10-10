use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: PoS.cls.ltxml — Proceedings of Science style.
  // LoadClass("JHEP", withoptions => 1);
  // …without JHEP's journal-citation macros, which PoS.cls does not define (jhep_cls.rs
  // `def_jhep_journal_macros`; witnesses 1507.03892, 1510.07042, 1807.03003, 1908.00100).
  AssignValue!("JHEP@without@journal@macros" => true);
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
    "\\lx@add@frontmatter{ltx:note}[role=conference]{#1}");

  DefMacro!("\\Jmath", "J");
  // PoS.cls v1.2.1 (2006/05/19, as 1507.03892 ships it):1384: a link to a PoS contribution by its
  // identifier (1411.6666, 0906.1503 cite `\pos{PoS(INTEGRAL08)029}`).
  DefMacro!("\\pos{}",
    "\\href{http://pos.sissa.it/cgi-bin/reader/contribution.cgi?id=#1}{\\tt #1}");
});
