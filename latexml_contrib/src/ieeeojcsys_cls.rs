//! Stub for IEEEojcsys.cls (IEEE Open Journal of Control Systems).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("hyperref");
  RequirePackage!("authblk");
  // IEEEojcsys.cls:3346-3348 `\usepackage{color,ifpdf,ifthen}\AtBeginDocument{\definecolor{ojcolor}{rgb}{0,0.40,0.80}}`
  // and the caption fonts :4669-4672, which papers name in their caption setup (`\captionsetup{labelfont={color=ojcolor}}`,
  // 2206.03535 — an Error per caption once caption's keys took effect, 68h; pdflatex 0).
  RequirePackage!("color");
  RequirePackage!("ifpdf");
  RequirePackage!("ifthen");
  RawTeX!(
    r"\AtBeginDocument{\definecolor{ojcolor}{rgb}{0,0.40,0.80}}%
\def\tablecapfont{\fontfamily{\sfdefault}\fontsize{8}{10}\selectfont\bfseries\rightskip0pt plus1fill}%
\def\tablecapnumfont{\fontfamily{\sfdefault}\fontsize{7}{9}\selectfont\bfseries}%
\def\figcapfont{\fontfamily{\sfdefault}\fontsize{7}{9}\selectfont\bfseries\rightskip0pt plus1fill}%
\def\figcapnumfont{\fontfamily{\sfdefault}\fontsize{7}{9}\selectfont\bfseries}%"
  );

  // Frontmatter — preserve author content.
  DefMacro!(
    "\\paper{}",
    "\\@add@frontmatter{ltx:note}[role=paper-type]{#1}"
  );
  DefMacro!("\\jvol{}", "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\jnum{}", "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!("\\jmonth{}", "\\@add@frontmatter{ltx:note}[role=month]{#1}");
  DefMacro!(
    "\\corresp{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  DefMacro!(
    "\\authornote{}",
    "\\@add@frontmatter{ltx:note}[role=authornote]{#1}"
  );
  DefMacro!(
    "\\receiveddate{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}"
  );
  DefMacro!("\\affilmark[]{}", "\\textsuperscript{#2}");

  // \appendices is an IEEE-style appendix env.
  DefMacro!("\\appendices", "\\appendix");

  // {IEEEkeywords} env.
  DefEnvironment!(
    "{IEEEkeywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>"
  );
});
