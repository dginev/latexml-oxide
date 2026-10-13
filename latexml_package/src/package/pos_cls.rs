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
  // PoS.cls loads color (v1.2.1 :174, v1.2.4 and v1.2.6 :162), so a paper's `\textcolor`, `\color` and
  // `\definecolor` need no package of its own (witnesses 1904.06989, 1309.7883, 1411.7592, 1708.03575,
  // 1811.03840). Perl's PoS.cls.ltxml:24-27 loads ifpdf, times, mathptmx and graphicx only (KNOWN_PERL_ERRORS #629).
  RequirePackage!("color");

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
  // PoS.cls (0711.1871's copy :1395, as every sampled copy at :1358-1395): `\PoS{PoS(LAT2009)123}` sets the
  // contribution's identifier (`\PoSsplit@ident`), which the class prints as the grey watermark down the page edge,
  // then `\let\PoS\gobble` (`\gobble`, which no copy defines, so a second call is undefined): the identifier as a
  // pubnote. Run 340: 79 of 120 sampled PoS papers call it, ~360
  // first errors (0711.1871, 0806.0754, 1012.3355).
  RawTeX!(r"\newcommand\PoS[1]{\lx@add@pubnote[role=pubid]{#1}\let\PoS\gobble}");
});
