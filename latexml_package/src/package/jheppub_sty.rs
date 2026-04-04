use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: jheppub.sty.ltxml — 112 lines
  RequirePackage!("hyperref");
  RequirePackage!("color");
  RequirePackage!("natbib");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("epsfig");
  RequirePackage!("graphicx");
  RequirePackage!("inst_support");

  // Author — Perl L32-34
  DefMacro!("\\author[]{}",
    "\\ifx.#1.\\else\\@institutemark{#1}\\fi\\def\\@author{#2}\\lx@author{#2}");

  // Affiliation — Perl L36-38
  DefConstructor!("\\affiliation[]{}",
    "<ltx:note role='institutetext' mark='#1'>#2</ltx:note>", bounded => true);

  // Footnote alias — Perl L41
  Let!("\\note", "\\footnote");

  // Email — Perl L43-44
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\emailAdd Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");

  // Keywords — Perl L46-47
  DefMacro!("\\keywordname", "\\textbf{Keywords}");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}[name={\\keywordname}]{#1}");

  // Frontmatter metadata — Perl L49-54
  DefMacro!("\\arxivnumber{}", "\\@add@frontmatter{ltx:note}[role=arxiv]{#1}");
  DefMacro!("\\preprint{}", "\\@add@frontmatter{ltx:note}[role=preprint]{#1}");
  DefMacro!("\\proceeding{}", "\\@add@frontmatter{ltx:note}[role=proceeding]{#1}");
  DefMacro!("\\dedicated{}", "\\@add@frontmatter{ltx:note}[role=dedication]{#1}");
  DefMacro!("\\collaboration{}{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@collaborator{#2}}");
  DefMacro!("\\collaborationImg[]{}", "");

  // Acknowledgements — Perl L56-60
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements>");
  DefConstructor!("\\endacknowledgments", "</ltx:acknowledgements>");
  Tag!("ltx:acknowledgements", auto_close => true);

  // Conditionals — Perl L62-65
  DefConditional!("\\ifaffil");
  DefConditional!("\\ifnotoc");
  DefConditional!("\\ifemailadd");
  DefConditional!("\\iftoccontinuous");

  // Empty defaults — Perl L68-77
  DefMacro!("\\@subheader", "\\@empty");
  DefMacro!("\\@keywords", "\\@empty");
  DefMacro!("\\@abstract", "\\@empty");
  DefMacro!("\\@xtum", "\\@empty");
  DefMacro!("\\@dedicated", "\\@empty");
  DefMacro!("\\@arxivnumber", "\\@empty");
  DefMacro!("\\@collaboration", "\\@empty");
  DefMacro!("\\@collaborationImg", "\\@empty");
  DefMacro!("\\@proceeding", "\\@empty");
  DefMacro!("\\@preprint", "\\@empty");

  // Spacing macros — Perl L80-96
  DefMacro!("\\afterLogoSpace", "\\smallskip");
  DefMacro!("\\afterSubheaderSpace", "\\vskip3pt plus 2pt minus 1pt");
  DefMacro!("\\afterProceedingsSpace", "\\vskip21pt plus0.4fil minus15pt");
  DefMacro!("\\afterTitleSpace", "\\vskip23pt plus0.06fil minus13pt");
  DefMacro!("\\afterRuleSpace", "\\vskip23pt plus0.06fil minus13pt");
  DefMacro!("\\afterCollaborationSpace", "\\vskip3pt plus 2pt minus 1pt");
  DefMacro!("\\afterCollaborationImgSpace", "\\vskip3pt plus 2pt minus 1pt");
  DefMacro!("\\afterAuthorSpace", "\\vskip5pt plus4pt minus4pt");
  DefMacro!("\\afterAffiliationSpace", "\\vskip3pt plus3pt");
  DefMacro!("\\afterEmailSpace", "\\vskip16pt plus9pt minus10pt\\filbreak");
  DefMacro!("\\afterXtumSpace", "\\par\\bigskip");
  DefMacro!("\\afterAbstractSpace", "\\vskip16pt plus9pt minus13pt");
  DefMacro!("\\afterKeywordsSpace", "\\vskip16pt plus9pt minus13pt");
  DefMacro!("\\afterArxivSpace", "\\vskip3pt plus0.01fil minus10pt");
  DefMacro!("\\afterDedicatedSpace", "\\vskip0pt plus0.01fil");
  DefMacro!("\\afterTocSpace", "\\bigskip\\medskip");
  DefMacro!("\\afterTocRuleSpace", "\\bigskip\\bigskip");

  // Misc — Perl L99-109
  DefMacro!("\\beforetochook", "");
  DefMacro!("\\notoc", "");
  DefMacro!("\\compress", "");
  DefMacro!("\\correctionref{}{}{}", "");
  DefMacro!("\\jname", "JHEP");
  DefMacro!("\\subheader{}", "");
  DefMacro!("\\xtumfont{}", "\\textsc{#1}");
  Let!("\\oldthebibliography", "\\thebibliography");
  Let!("\\endoldthebibliography", "\\endthebibliography");
});
