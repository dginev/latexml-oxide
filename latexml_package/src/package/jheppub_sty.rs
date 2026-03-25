use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("hyperref");
  RequirePackage!("color");
  RequirePackage!("natbib");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("epsfig");
  RequirePackage!("graphicx");
  RequirePackage!("inst_support");
  DefMacro!("\\author[]{}",
    "\\ifx.#1.\\else\\@institutemark{#1}\\fi\\def\\@author{#2}\\lx@author{#2}");
  DefConstructor!("\\affiliation[]{}",
    "<ltx:note role='institutetext' mark='#1'>#2</ltx:note>");
  Let!("\\note", "\\footnote");
  DefConstructor!("\\@@@email{}",
    "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\emailAdd Semiverbatim",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefMacro!("\\keywordname",  "\\textbf{Keywords}");
  DefMacro!("\\keywords{}",   "\\@add@frontmatter{ltx:keywords}[name={\\keywordname}]{#1}");
  DefMacro!("\\arxivnumber{}",        "\\@add@frontmatter{ltx:note}[role=arxiv]{#1}");
  DefMacro!("\\preprint{}",           "\\@add@frontmatter{ltx:note}[role=preprint]{#1}");
  DefMacro!("\\proceeding{}",         "\\@add@frontmatter{ltx:note}[role=proceeding]{#1}");
  DefMacro!("\\dedicated{}",          "\\@add@frontmatter{ltx:note}[role=dedication]{#1}");
  DefMacro!("\\collaboration{}{}",    "\\@add@to@frontmatter{ltx:creator}{\\@@@collaborator{#2}}");
  DefMacro!("\\collaborationImg[]{}", "");
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  DefConditional!("\\ifaffil");
  DefConditional!("\\ifnotoc");
  DefConditional!("\\ifemailadd");
  DefConditional!("\\iftoccontinuous");
  DefMacro!("\\beforetochook",  "");
  DefMacro!("\\notoc",          "");
  DefMacro!("\\compress",       "");
  DefMacro!("\\jname",              "JHEP");
  DefMacro!("\\subheader{}",        "");
  DefMacro!("\\xtumfont{}",         "\\textsc{#1}");
  Let!("\\oldthebibliography",    "\\thebibliography");
  Let!("\\endoldthebibliography", "\\endthebibliography");
});
