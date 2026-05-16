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

  // Author-block font switches: no-op (identity).
  DefMacro!("\\name", "");
  DefMacro!("\\addr", "");
  DefMacro!("\\email", "");
  DefMacro!("\\And", " ");

  // Frontmatter / pagination ceremony — gobble cleanly.
  DefMacro!("\\jmlrheading{}{}{}{}{}{}", "");
  DefMacro!("\\ShortHeadings{}{}", "");
  DefMacro!("\\firstpageno{}", "");
  DefMacro!("\\editor{}", "");
  DefMacro!("\\editors{}", "");

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
