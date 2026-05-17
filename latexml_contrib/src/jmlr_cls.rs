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
  // JMLR frontmatter — preserve author-typed metadata as ltx:note so
  // it reaches the XML (content-preserving). Year/page/workshop/dates
  // are short scalars but the editor list is real prose authors care
  // about; gobbling drops attribution.
  DefMacro!("\\jmlryear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\jmlrworkshop{}",
    "\\@add@frontmatter{ltx:note}[role=workshop]{#1}");
  DefMacro!("\\jmlrsubmitted{}",
    "\\@add@frontmatter{ltx:note}[role=submitted]{#1}");
  DefMacro!("\\jmlrpublished{}",
    "\\@add@frontmatter{ltx:note}[role=published]{#1}");
  DefMacro!("\\jmlrproceedings{}{}",
    "\\@add@frontmatter{ltx:note}[role=proceedings]{#1: #2}");
  DefMacro!("\\editor{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}");
  DefMacro!("\\editors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}");
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
