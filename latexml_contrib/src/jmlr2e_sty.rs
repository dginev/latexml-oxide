//! Stub for jmlr2e.sty / jmlr2e_preprint.sty (JMLR / PMLR author-block macros).
//!
//! These styles are used by Journal of Machine Learning Research and
//! related Proceedings of Machine Learning Research papers. Define the
//! \name / \addr / \email author-block font switches as identity so
//! `\author{\name Foo \email a@b.c \\ \addr Place}` parses cleanly.
use latexml_package::{
  prelude::*,
  shipped_class::{class_code, shipped_copy_text},
};

LoadDefinitions!({
  RequirePackage!("natbib");
  // jmlr2e.sty does not load amsthm; it defines `{proof}` itself (jmlr2e.sty of 2022-10-17,
  // :383-387 `\ifdefined\proof\renewenvironment{proof}…\else\newenvironment{proof}…\fi`). The binding borrows amsthm's
  // proof and theorem machinery (`require_package_borrowed`): `\@ifpackageloaded{amsthm}` stays false, so a
  // paper's `\let\proof\relax \let\endproof\relax` followed by its own `\usepackage{amsthm}` loads amsthm,
  // whose `{proof}` then takes over — loaded outright, the second request was skipped and `{proof}` stayed
  // `\relax` (40 of the 43 `undefined:{proof}` jmlr2e papers; 1302.6584, 1310.4849, 1404.0099). The proof is
  // jmlr2e's own (`claim_borrowed_meanings`), as the real file defines it.
  require_package_borrowed("amsthm")?;
  claim_borrowed_meanings(&["\\proof", "\\endproof", "\\begin{proof}", "\\end{proof}"]);
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
  // (a space; a `" "` string body would tokenize as an empty line, `\par`: KNOWN_PERL_ERRORS #984)
  DefMacro!(T_CS!("\\And"), None, Tokens!(T_SPACE!()));

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
  DefMacro!(
    "\\jmlrheading{}{}{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=heading]{#1 #2 #3 #4 #5 #6}"
  );
  // jmlr2e.sty L256: `\def\ewrlheading#1#2#3#4{…\def\ps@jmlrtps{…}…}` — the
  // EWRL-proceedings variant of `\jmlrheading`, setting the running-head page
  // style from {volume}{year}{date/location}{authors}. Our binding intercepts
  // jmlr2e.sty (so the raw def never runs); Perl raw-loads it and is clean.
  // Preserve the proceedings metadata as an ltx:note, same as `\jmlrheading`
  // (HTML drops the running head, so the raw def would lose this content).
  // Witness 1802.03976 (`\ewrlheading{14}{2018}{October 2018, Lille, France}
  // {…authors…}`).
  DefMacro!(
    "\\ewrlheading{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=heading]{#1 #2 #3 #4}"
  );
  DefMacro!(
    "\\ShortHeadings{}{}",
    "\\@add@frontmatter{ltx:note}[role=shortheadings]{#1 / #2}"
  );
  DefMacro!(
    "\\firstpageno{}",
    "\\@add@frontmatter{ltx:note}[role=firstpage]{#1}"
  );
  // \editor / \editors carry author-supplied editor names — preserve as
  // ltx:note rather than dropping. JMLR papers cite the editor in the
  // header; this keeps the credit visible.
  DefMacro!(
    "\\editor{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}"
  );
  DefMacro!(
    "\\editors{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}"
  );
  // jmlr2e.sty L194: `\def\address#1{\gdef\@address{#1}}` — the author's
  // institutional address, rendered in the title block (`\@name \\ \@address`,
  // L187). Our binding intercepts jmlr2e.sty (so the raw def never runs), and
  // OmniBus only autoloads `\address` for revtex/OmniBus contexts — so a
  // jmlr2e paper using `\address{…}` left it undefined where Perl (raw-loads
  // jmlr2e) is clean. Preserve the address content as an ltx:note, consistent
  // with the other jmlr2e frontmatter macros above. Witness 1711.01660.
  DefMacro!(
    "\\address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}"
  );

  // jmlr2e.sty's `\acks{text}` (2511.10879's copy :372, 1512.04829's :289): the acknowledgments under the heading the
  // copy prints, `{\large\bf Acknowledgments and Disclosure of Funding}` in the newer copies, `{\large\bf
  // Acknowledgments}` in the older ones — read from the paper's copy. Without a copy, "Acknowledgments": the heading of
  // 45 of the 64 copies arXiv papers ship (30,840 sampled; the other 19 the longer one, most of them 2023-25), and
  // jmlr.cls:595's. The binding wrote the slug "acknowledgments-disclosure-of-funding", which the XSLT printed as the
  // heading. Guard `perfect_kernel_batch78c::jmlr_acknowledgements_carry_the_printed_heading`.
  let heading = shipped_copy_text("sty")?
    .and_then(|text| acks_heading(&class_code(&text)))
    .unwrap_or_else(|| "Acknowledgments".to_string());
  def_macro(
    T_CS!("\\lx@jmlr@acksname"),
    None,
    Some(ExpansionBody::from(heading)),
    None,
  )?;
  DefConstructor!("\\acks{}", "<ltx:acknowledgements name='#name'>#1</ltx:acknowledgements>",
  properties => sub[_args] {
    Ok(stored_map!("name" => Stored::from(digest_in_group(T_CS!("\\lx@jmlr@acksname"))?)))
  });

  // jmlr2e.sty L33-36: the camera-ready toggle.
  //   \newif\ifnipsfinal \nipsfinalfalse
  //   \def\nipsfinalcopy{\nipsfinaltrue}
  // Papers uncomment `\nipsfinalcopy` for the camera-ready build (it gates
  // submission-notice / page-number formatting in the raw .sty). Our binding
  // intercepts jmlr2e.sty so the raw def never runs; Perl raw-loads it and is
  // clean. Mirror the real definitions — the gated formatting is layout-only
  // (moot in our XML paradigm), so the conditional just needs to exist.
  // Witness 1604.06057 (`\nipsfinalcopy` in main.tex).
  RawTeX!(r"\newif\ifnipsfinal\nipsfinalfalse\def\nipsfinalcopy{\nipsfinaltrue}");

  // {keywords} env — frontmatter list, render as classification block.
  DefEnvironment!(
    "{keywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>"
  );
  // jmlr2e.sty:151-153 defines only the environment, whose begin code (a group, "Keywords:") a
  // `\keywords{A, B}` written as a command runs: pdflatex prints the braced list after the
  // label and leaves the group open to the end of the document (a warning). Here the command
  // form is the environment around its argument; the open classification otherwise swallowed
  // the rest of the document (1401.6686).
  Let!("\\lx@jmlr@keywords@begin", "\\keywords");
  DefMacro!(
    "\\keywords",
    "\\@ifnextchar\\bgroup\\lx@jmlr@keywords@arg\\lx@jmlr@keywords@begin"
  );
  DefMacro!(
    "\\lx@jmlr@keywords@arg{}",
    "\\begin{keywords}#1\\end{keywords}"
  );

  // Theorem-likes JMLR papers commonly use.
  // jmlr2e.sty of 2022-10-17 :388-396 (unchanged since the 2001 file): `example` has a counter of its own, the rest share
  // `theorem`'s; `conjecture` and `axiom` were missing (18 and 4 papers; 0907.5309, 1108.0065).
  RawTeX!(
    r"\newtheorem{example}{Example}
\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{proposition}[theorem]{Proposition}
\newtheorem{remark}[theorem]{Remark}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{definition}[theorem]{Definition}
\newtheorem{conjecture}[theorem]{Conjecture}
\newtheorem{axiom}[theorem]{Axiom}"
  );
});

/// The heading a jmlr2e.sty copy's `\acks` prints: the text of its bold group, `\long\def\acks#1{\vskip 0.3in\noindent
/// {\large\bf Acknowledgments}…`.
fn acks_heading(code: &str) -> Option<String> {
  let body = &code[code.find("\\def\\acks")?..];
  let body = &body[..body.find("#1}").map_or(body.len(), |end| end + 3)];
  let text = &body[body.find("\\bf")? + 3..];
  let text = text[..text.find('}')?].trim();
  (!text.is_empty()).then(|| text.to_string())
}
