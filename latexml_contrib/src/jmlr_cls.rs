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
  // jmlrutils.sty L86-140: reference helpers. Stub as the LaTeX
  // \ref expansion so cross-refs still resolve. Witness 2409.07012.
  DefMacro!("\\sectionref{}", "\\ref{#1}");
  DefMacro!("\\appendixref{}", "\\ref{#1}");
  DefMacro!("\\equationref{}", "(\\ref{#1})");
  DefMacro!("\\theoremref{}", "\\ref{#1}");
  DefMacro!("\\lemmaref{}", "\\ref{#1}");
  DefMacro!("\\corollaryref{}", "\\ref{#1}");
  DefMacro!("\\propositionref{}", "\\ref{#1}");
  DefMacro!("\\definitionref{}", "\\ref{#1}");
  DefMacro!("\\exampleref{}", "\\ref{#1}");
  DefMacro!("\\remarkref{}", "\\ref{#1}");

  // jmlrutils theorem-style configuration helpers (gobble silently —
  // we don't replicate the punctuation/spacing). Witness: 2502.19625
  // (\theorempostheader{:}).
  DefMacro!("\\theorempostheader{}", "");
  DefMacro!("\\theoremheader{}", "");
  DefMacro!("\\theoremsep{}", "");
  DefMacro!("\\theoremprework{}", "");
  DefMacro!("\\theorempostwork{}", "");
  DefMacro!("\\theorembodyfont{}", "");
  DefMacro!("\\theoremheaderfont{}", "");
  DefMacro!("\\definetheoremstyle{}{}", "");
  DefMacro!("\\settheoremtag{}", "");

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
