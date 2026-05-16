//! Stub for jmlr.cls and clear2025.cls family.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("natbib");

  // Author-block primitives (jmlr.cls L335-342, L374-445).
  DefMacro!("\\addr", "");
  DefMacro!("\\Name[]{}", "#2");
  DefMacro!("\\Email{}", "");
  DefMacro!("\\IncludeName{}{}", "");
  DefMacro!("\\And", " ");
  DefMacro!("\\acks{}", "");
  DefMacro!("\\clearauthor{}", "\\author{#1}");

  // Frontmatter / pagination ceremony.
  DefMacro!("\\jmlrheading{}{}{}{}{}{}", "");
  DefMacro!("\\jmlrvolume{}", "");
  DefMacro!("\\jmlryear{}", "");
  DefMacro!("\\jmlrworkshop{}", "");
  DefMacro!("\\jmlrsubmitted{}", "");
  DefMacro!("\\jmlrpublished{}", "");
  DefMacro!("\\jmlrproceedings{}{}", "");
  DefMacro!("\\editor{}", "");
  DefMacro!("\\editors{}", "");
  DefMacro!("\\firstpageno{}", "");

  // {keywords} env.
  DefEnvironment!(
    "{keywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>"
  );

  // jmlrcombine helpers used in tables / floats.
  DefMacro!("\\floatconts{}{}{}", "#3");
  DefMacro!("\\tableref{}", "#1");
  DefMacro!("\\figureref{}", "#1");
  DefMacro!("\\algorithmref{}", "#1");

  // Theorem-likes.
  RawTeX!(
    r"\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{proposition}[theorem]{Proposition}
\newtheorem{definition}[theorem]{Definition}
\newtheorem{example}[theorem]{Example}
\newtheorem{remark}[theorem]{Remark}"
  );
});
