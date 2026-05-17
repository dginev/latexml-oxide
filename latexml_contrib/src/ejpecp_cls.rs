//! Stub for ejpecp.cls (Electronic Journal of Probability / Communications in Probability).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // ejpecp frontmatter (L118-129) — preserve author content.
  DefMacro!("\\TITLE{}", "\\title{#1}");
  DefMacro!("\\SHORTTITLE{}",
    "\\@add@frontmatter{ltx:note}[role=shorttitle]{#1}");
  DefMacro!("\\KEYWORDS{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\AMSSUBJ{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#1}");
  DefMacro!("\\ABSTRACT{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\AUTHORS{}", "\\author{#1}");
  DefMacro!("\\VOLUME{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\PAPERNUM{}",
    "\\@add@frontmatter{ltx:note}[role=papernumber]{#1}");
  DefMacro!("\\YEAR{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\SUBMITTED{}",
    "\\@add@frontmatter{ltx:note}[role=submitted]{#1}");
  DefMacro!("\\ACCEPTED{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1}");
  DefMacro!("\\DOI{}",
    "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\EMAIL{}",
    "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  DefMacro!("\\support{}",
    "\\@add@frontmatter{ltx:note}[role=support]{#1}");

  // Standard envs commonly used in probability papers.
  DefEnvironment!("{acks}", "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  RawTeX!(r"\newtheorem{theorem}{Theorem}");
  RawTeX!(r"\newtheorem{lemma}[theorem]{Lemma}");
  RawTeX!(r"\newtheorem{proposition}[theorem]{Proposition}");
  RawTeX!(r"\newtheorem{corollary}[theorem]{Corollary}");
  RawTeX!(r"\newtheorem{definition}[theorem]{Definition}");
  RawTeX!(r"\newtheorem{remark}[theorem]{Remark}");
  RawTeX!(r"\newtheorem{example}[theorem]{Example}");
  RawTeX!(r"\newtheorem{assumption}[theorem]{Assumption}");
});
