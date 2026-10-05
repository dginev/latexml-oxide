//! Stub for lmcs.cls (Logical Methods in Computer Science journal class).
//!
//! lmcs.cls is NOT distributed in TeX Live (papers bundle it, but the corpus
//! copies don't reach our search path), so Perl LaTeXML — which ships no lmcs
//! binding — emits `Can't find binding for class lmcs (using OmniBus)` and
//! falls back to **OmniBus**. We load OmniBus as the base (NOT amsart, which the
//! prior version used), and the class's own theorem set (lmcs.cls:696-761, below)
//! eagerly: a paper whose preamble does `\newtheorem{remark}[thm]{…}` and whose
//! body uses `\begin{thm}`/`\begin{lem}` shares the class's `thm` counter,
//! numbered within the section as pdflatex numbers it (OmniBus's lazy autoloads
//! once supplied `thm`/`lem` with document-wide numbers). Witness 1607.01886. The raw cls relies on pgfmath/tikz/XeTeXLinkBox for
//! ORCID-logo rendering (fails mid-load), so `\lmcsdoi`/`\lmcsheading`/
//! `\lmcsorcid` get content-preserving stubs below. Witness 2305.14448,
//! 2305.19985, 1607.04128 (graphicx).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  // lmcs.cls L33 `\LoadClass[11pt,reqno]{amsart}`. Perl ships no lmcs binding,
  // so it loads OmniBus AND dependency-scans the raw lmcs.cls, which loads
  // amsart.cls.ltxml → ams_support.sty.ltxml. That is where the amsart-family
  // frontmatter macros (`\urladdr`, `\address`, `\email`, `\curraddr`) come
  // from. Without it, `\urladdr{\url{…}}` was undefined (witness 1709.06170,
  // RUST 1 → 0; Perl loads it via the amsart dep). ams_support is frontmatter
  // only — it does not pre-declare theorem envs; those come from the class's
  // theorem set below (1607.01886).
  RequirePackage!("ams_support");
  RequirePackage!("hyperref");
  // lmcs.cls L24 `\usepackage{helvet,cclicenses}` pulls graphicx in
  // transitively: cclicenses → `\RequirePackage{rotating}` (cclicenses.sty
  // L25) → `\RequirePackage{graphicx}` (rotating.sty L62). Perl ships no lmcs
  // binding, so it raw-loads lmcs.cls and gets graphicx that way; LMCS papers
  // therefore use `\includegraphics` WITHOUT their own `\usepackage{graphicx}`.
  // Our stub intercepts the raw cls (which fails mid-load on its tikz/
  // XeTeXLinkBox ORCID machinery), so the transitive graphicx never loaded
  // and `\includegraphics` was undefined where Perl is clean. Supply it
  // directly. Witness 1607.04128 (`\documentclass{lmcs}`, `\includegraphics`
  // with no explicit graphicx load): RUST 1 → 0.
  RequirePackage!("graphicx");
  // xcolor comes with tikz (pgfcore.sty:13); a paper's later `\usepackage[table]{xcolor}` still takes its options (a
  // repeat load applies the options the first lacked, content.rs `apply_new_options_on_reload`), so colortbl and
  // array `m{}`/`b{}` columns work. See ifacconf_cls.rs.
  RequirePackage!("enumitem");
  RequirePackage!("etoolbox");
  // lmcs.cls:62-66, 399, 428: tikz (papers draw with it unloaded, 2609.11893), color, thmtools, xparse, mathtools
  // (`\coloneqq`, `\DeclarePairedDelimiter`).
  RequirePackage!("tikz");
  RequirePackage!("color");
  RequirePackage!("thmtools");
  RequirePackage!("xparse");
  RequirePackage!("mathtools");
  // lmcs.cls:696-761: the class's theorem environments, numbered together by section (`{exa}`, `{defi}` were
  // undefined, 2609.11893), with its `defC`/`thmC` styles for theorems carrying a citation (:731-758).
  RawTeX!(
    r"\theoremstyle{plain}
\newtheorem{thm}{Theorem}[section]\newtheorem{cor}[thm]{Corollary}\newtheorem{lem}[thm]{Lemma}
\newtheorem{slem}[thm]{Sublemma}\newtheorem{prop}[thm]{Proposition}
\theoremstyle{definition}
\newtheorem{asm}[thm]{Assumption}\newtheorem{rem}[thm]{Remark}\newtheorem{rems}[thm]{Remarks}
\newtheorem{exa}[thm]{Example}\newtheorem{exas}[thm]{Examples}\newtheorem{defi}[thm]{Definition}
\newtheorem{conv}[thm]{Convention}\newtheorem{conj}[thm]{Conjecture}\newtheorem{prob}[thm]{Problem}
\newtheorem{oprob}[thm]{Open Problem}\newtheorem{oprobs}[thm]{Open Problems}\newtheorem{algo}[thm]{Algorithm}
\newtheorem{obs}[thm]{Observation}\newtheorem{desc}[thm]{Description}\newtheorem{fact}[thm]{Fact}
\newtheorem{qu}[thm]{Question}\newtheorem{oqu}[thm]{Open Question}\newtheorem{pty}[thm]{Property}
\newtheorem{clm}[thm]{Claim}\newtheorem{nota}[thm]{Notation}\newtheorem{com}[thm]{Comment}
\newtheorem{coms}[thm]{Comments}
\newtheoremstyle{defC}{6pt}{6pt}{\normalfont}{}{\bfseries}{{\bfseries .}}{5pt plus 1pt minus 1pt}{\thmname{#1} \thmnumber{#2} \thmnote{\normalfont#3}}
\theoremstyle{defC}
\newtheorem{defiC}[thm]{Definition}\newtheorem{remC}[thm]{Remark}\newtheorem{exaC}[thm]{Example}
\newtheoremstyle{thmC}{6pt}{6pt}{\itshape}{}{\bfseries}{{\bfseries .}}{5pt plus 1pt minus 1pt}{\thmname{#1} \thmnumber{#2} \thmnote{\normalfont#3}}
\theoremstyle{thmC}
\newtheorem{thmC}[thm]{Theorem}\newtheorem{propC}[thm]{Proposition}\newtheorem{lemC}[thm]{Lemma}
\theoremstyle{plain}
\numberwithin{equation}{section}"
  );

  // LMCS publication metadata. Real macros assign internal counters
  // and set up running headers; for HTML rendering we just preserve
  // the args as named notes.
  DefMacro!(
    "\\lmcsdoi{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=lmcs-doi]{Volume #1, Issue #2, Paper #3}"
  );
  // \lmcsheading{vol}{issue}{year}{pages}{subm}{publ}{rev}{spec_iss}{title}
  // The raw cls signature is 7-args but with optional/positional variations.
  // We don't reproduce the running-header layout — just discard, since the
  // metadata is already captured by \lmcsdoi.
  def_macro_noop("\\lmcsheading{}{}{}{}")?;
  // \lmcsorcid{orcid-id} — render as a plain link rather than the
  // tikz/XeTeXLinkBox logo construction.
  DefMacro!("\\lmcsorcid{}", "\\href{https://orcid.org/#1}{ORCID:#1}");

  // Section-numbering and shortauthors/shorttitle helpers used by raw
  // cls header layout. Stub as no-op or pass-through.
  def_macro_noop("\\shorttitle{}")?;
  def_macro_noop("\\shortauthors{}")?;

  // `\dOi` placeholder produced by the raw cls when no \lmcsdoi was
  // declared. Stub to empty so it doesn't appear as red error text.
  def_macro_noop("\\dOi")?;
});
