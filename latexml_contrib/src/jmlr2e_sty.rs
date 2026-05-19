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
  def_macro_noop("\\name")?;
  def_macro_noop("\\addr")?;
  def_macro_noop("\\email")?;
  DefMacro!("\\And", " ");

  // Frontmatter / pagination ceremony. Round-34 surpass-Perl:
  // preserve the author-typed text content (volume/page/etc. tuples
  // and running-head author+title pair) as ltx:note rather than
  // dropping silently.
  DefMacro!("\\jmlrheading{}{}{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=heading]{#1 #2 #3 #4 #5 #6}");
  DefMacro!("\\ShortHeadings{}{}",
    "\\@add@frontmatter{ltx:note}[role=shortheadings]{#1 / #2}");
  DefMacro!("\\firstpageno{}",
    "\\@add@frontmatter{ltx:note}[role=firstpage]{#1}");
  // \editor / \editors carry author-supplied editor names — preserve as
  // ltx:note rather than dropping. JMLR papers cite the editor in the
  // header; this keeps the credit visible.
  DefMacro!("\\editor{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}");
  DefMacro!("\\editors{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}");

  // jmlr2e.sty L372: \acks{text} — acknowledgments section. Emit as
  // structural ltx:acknowledgements with the funding-disclosure label
  // (post-processors map to canonical role/styling).
  DefConstructor!("\\acks{}",
    "<ltx:acknowledgements name='acknowledgments-disclosure-of-funding'>#1</ltx:acknowledgements>");

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
