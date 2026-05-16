//! Stub for jmlr2e.sty / jmlr2e_preprint.sty (JMLR / PMLR author-block macros).
//!
//! These styles are used by Journal of Machine Learning Research and
//! related Proceedings of Machine Learning Research papers. Define the
//! \name / \addr / \email author-block font switches as identity so
//! `\author{\name Foo \email a@b.c \\ \addr Place}` parses cleanly.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("natbib");
  RequirePackage!("amsthm");
  // jmlr2e.sty L57-63 pulls in epsfig, amssymb, graphicx, hyperref.
  // Mirror that so user code that calls \hypersetup / \href / \blacklozenge
  // (from amssymb) at preamble time doesn't error. Witness 2406.03260.
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");

  // Author-block font switches: no-op (identity).
  DefMacro!("\\name", "");
  DefMacro!("\\addr", "");
  DefMacro!("\\email", "");
  DefMacro!("\\And", " ");

  // Frontmatter / pagination ceremony.
  DefMacro!("\\jmlrheading{}{}{}{}{}{}", "");
  DefMacro!("\\ShortHeadings{}{}", "");
  DefMacro!("\\firstpageno{}", "");
  // \editor / \editors carry author-supplied editor names — preserve as
  // ltx:note rather than dropping. JMLR papers cite the editor in the
  // header; this keeps the credit visible.
  DefMacro!("\\editor{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}");
  DefMacro!("\\editors{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}");

  // jmlr2e.sty L372: \acks{text} — acknowledgments section. Render as a
  // section heading so the text body still appears.
  DefMacro!("\\acks{}", "\\section*{Acknowledgments and Disclosure of Funding}#1");

  // {keywords} env — frontmatter list, render as classification block.
  DefEnvironment!(
    "{keywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>"
  );

  // Theorem-likes JMLR papers commonly use.
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
