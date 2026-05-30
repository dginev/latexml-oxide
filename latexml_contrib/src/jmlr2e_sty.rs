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

  // jmlr2e.sty: `\newcommand{\BlackBox}{\rule{1.5ex}{1.5ex}}` — the
  // end-of-proof QED box. Papers use `\hfill\BlackBox` or
  // `\def\qed{\hfill\BlackBox}`. The stub previously omitted it, so a
  // JMLR paper that loads jmlr2e and ends proofs with `\BlackBox` saw it
  // undefined (witness 2001.10284: the only residual error after the
  // stub handled \editor/{keywords}/\ShortHeadings/\firstpageno). Mirror
  // the real definition.
  DefMacro!("\\BlackBox", "\\rule{1.5ex}{1.5ex}");

  // Frontmatter / pagination ceremony. Round-34 surpass-Perl:
  // preserve the author-typed text content (volume/page/etc. tuples
  // and running-head author+title pair) as ltx:note rather than
  // dropping silently.
  DefMacro!("\\jmlrheading{}{}{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=heading]{#1 #2 #3 #4 #5 #6}");
  // jmlr2e.sty L256: `\def\ewrlheading#1#2#3#4{…\def\ps@jmlrtps{…}…}` — the
  // EWRL-proceedings variant of `\jmlrheading`, setting the running-head page
  // style from {volume}{year}{date/location}{authors}. Our binding intercepts
  // jmlr2e.sty (so the raw def never runs); Perl raw-loads it and is clean.
  // Preserve the proceedings metadata as an ltx:note, same as `\jmlrheading`
  // (HTML drops the running head, so the raw def would lose this content).
  // Witness 1802.03976 (`\ewrlheading{14}{2018}{October 2018, Lille, France}
  // {…authors…}`).
  DefMacro!("\\ewrlheading{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=heading]{#1 #2 #3 #4}");
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
  // jmlr2e.sty L194: `\def\address#1{\gdef\@address{#1}}` — the author's
  // institutional address, rendered in the title block (`\@name \\ \@address`,
  // L187). Our binding intercepts jmlr2e.sty (so the raw def never runs), and
  // OmniBus only autoloads `\address` for revtex/OmniBus contexts — so a
  // jmlr2e paper using `\address{…}` left it undefined where Perl (raw-loads
  // jmlr2e) is clean. Preserve the address content as an ltx:note, consistent
  // with the other jmlr2e frontmatter macros above. Witness 1711.01660.
  DefMacro!("\\address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}");

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
