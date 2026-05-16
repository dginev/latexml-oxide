//! Stub for ejpecp.cls (Electronic Journal of Probability / Communications in Probability).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // ejpecp frontmatter (L118-129).
  DefMacro!("\\TITLE{}", "\\title{#1}");
  DefMacro!("\\SHORTTITLE{}", "");
  DefMacro!("\\KEYWORDS{}", "");
  DefMacro!("\\AMSSUBJ{}", "");
  DefMacro!("\\ABSTRACT{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\AUTHORS{}", "\\author{#1}");
  DefMacro!("\\VOLUME{}", "");
  DefMacro!("\\PAPERNUM{}", "");
  DefMacro!("\\YEAR{}", "");
  DefMacro!("\\SUBMITTED{}", "");
  DefMacro!("\\ACCEPTED{}", "");
  DefMacro!("\\DOI{}", "");
  DefMacro!("\\EMAIL{}", "");
  DefMacro!("\\support{}", "");

  // Standard envs commonly used in probability papers.
  DefEnvironment!("{acks}", "<ltx:acknowledgements>#body</ltx:acknowledgements>");
  RawTeX!(r"\newtheorem{theorem}{Theorem}");
  RawTeX!(r"\newtheorem{lemma}[theorem]{Lemma}");
  RawTeX!(r"\newtheorem{proposition}[theorem]{Proposition}");
  RawTeX!(r"\newtheorem{corollary}[theorem]{Corollary}");
  RawTeX!(r"\newtheorem{definition}[theorem]{Definition}");
  RawTeX!(r"\newtheorem{remark}[theorem]{Remark}");
  RawTeX!(r"\newtheorem{example}[theorem]{Example}");
  RawTeX!(r"\newtheorem{assumption}[theorem]{Assumption}");
});
