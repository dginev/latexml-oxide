//! paspconf.sty — the PASP conference-proceedings substyle of AAS RevTeX 3 (v1.6, 12 Aug 93:
//! `\revtex@org{PASP}`, `\revtex@genre{conference proceedings}`, L1-8), loaded as a
//! `\documentstyle[11pt,paspconf]{article}` option by ASP Conference Series papers. Not in
//! TeX Live; often not shipped either.
//!
//! It is aaspp.sty's sibling: the same RevTeX-3 frontmatter (`\affil` L72, `\altaffilmark`
//! L77, `\altaffiltext` L78), table notes (L85-104), `\acknowledgments` (L159-162),
//! `references` (L174-177), journal abbreviations (L195-227), astronomical symbols
//! (L228-254) and `\plotone`/`\plottwo`/`\plotfiddle` (L258-265) — so it loads the aaspp
//! binding (aas_support) and adds what is paspconf's own. Perl has no binding for the name
//! (its FindFile_fallback, Package.pm:2177-2189, strips no part of it), so a shipped copy is
//! raw-loaded and a missing one is an unmet `\documentstyle` option that loads OmniBus (its
//! `\affil`, a `\reference` keyed by the entry's first token, no `{answer}`); ours had mapped the
//! name to the unrelated ICASSP spconf binding, which defines no `\affil` (~860 papers of arXiv
//! run 336, 298 shipping the file). Witnesses astro-ph9712155, astro-ph9512088, astro-ph9605141.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("aaspp");
  // paspconf.sty:121-122 `\let\keywords=\@gobble`, `\let\subjectheadings=\@gobble`: the PDF
  // prints no keywords, so they are kept as metadata only (RDFa `dcterms:subject` through
  // hyperref's `pdfkeywords`, as icml_support_sty.rs), not as a displayed keywords block.
  DefMacro!("\\keywords{}", "\\hypersetup{pdfkeywords={#1}}");
  Let!("\\subjectheadings", "\\keywords");
  // paspconf.sty:41-42: line spacing; layout.
  def_macro_noop("\\loosenlines")?;
  // paspconf.sty:124-125: `\upper` makes `\sec@upcase` upper-case the titles; letter case is
  // typography, the titles keep the author's text.
  def_macro_noop("\\upper")?;
  RawTeX!(r"\def\sec@upcase#1{\relax#1}");
  // paspconf.sty:228 `\def\deg{\hbox{$^\circ$}}`, the degree sign (the kernel's `\deg` is the
  // operator).
  Let!("\\deg", "\\arcdeg");
  // paspconf.sty:163-170: the conference discussion — the first `question` opens an unnumbered
  // "Discussion" section; each question and answer starts with its speaker in italics.
  RawTeX!(r"\def\qanda@heading{Discussion}
\newif\if@firstquestion \@firstquestiontrue
\newenvironment{question}[1]{\if@firstquestion
\section*{\qanda@heading}\global\@firstquestionfalse\fi
\par\vskip 1ex
\noindent{\it#1\/}:}{\par}
\newenvironment{answer}[1]{\par\vskip 1ex
\noindent{\it#1\/}:}{\par}");
  // paspconf.sty:171-173: equations numbered within sections, `<section>-<equation>`.
  RawTeX!(r"\def\mathwithsecnums{\@addtoreset{equation}{section}%
\def\theequation{\arabic{section}-\arabic{equation}}}");
});
