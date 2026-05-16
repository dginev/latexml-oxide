//! Stub for sigma.cls (SIGMA journal — Symmetry, Integrability, Geometry).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // SIGMA frontmatter — preserve author content.
  DefMacro!("\\FirstPageHeading", "");
  DefMacro!("\\ShortArticleName{}",
    "\\@add@frontmatter{ltx:note}[role=shorttitle]{#1}");
  // \ArticleName{title} → \title{...}, \Author{name} → \author{...}.
  DefMacro!("\\ArticleName{}", "\\title{#1}");
  DefMacro!("\\Author{}", "\\author{#1}");
  DefMacro!("\\AuthorNameForHeading{}",
    "\\@add@frontmatter{ltx:note}[role=runningauthor]{#1}");
  DefMacro!("\\Address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}");
  DefMacro!("\\EmailD{}",
    "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  DefMacro!("\\URLaddressD{}",
    "\\@add@frontmatter{ltx:note}[role=url]{#1}");
  DefMacro!("\\ArticleDates{}",
    "\\@add@frontmatter{ltx:note}[role=dates]{#1}");
  // sigma.cls custom frontmatter macros for abstract / keywords /
  // classification + last-page sentinel.
  DefMacro!("\\Abstract{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\Keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\Classification{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#1}");
  DefMacro!("\\LastPageEnding", "");
});
