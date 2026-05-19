//! Stub for fundam.cls (Fundamenta Informaticae journal class).
//!
//! fundam.cls (v3.0, 2020) extends article for the Fundamenta Informaticae
//! journal. The raw cls defines `\publyear`, `\papernumber`, `\volume`,
//! `\issue` as simple metadata setters, but its preamble runs theorem.sty
//! and other env-heavy packages that fail mid-load, leaving the metadata
//! macros undefined. Witness 2305.16882.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("article");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  RequirePackage!("fancyhdr");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Article-metadata setters — raw cls assigns to internal `\@publyear`
  // etc.; HTML rendering surfaces as named frontmatter notes.
  DefMacro!("\\publyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\papernumber{}",
    "\\@add@frontmatter{ltx:note}[role=papernumber]{#1}");
  DefMacro!("\\volume{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\issue{}",
    "\\@add@frontmatter{ltx:note}[role=issue]{#1}");

  // \finalVersionForARXIV — toggles a `\finalarxivtrue` switch in raw
  // cls; HTML rendering ignores layout switches.
  DefMacro!("\\finalVersionForARXIV", "");
  DefConditional!("\\iffinalarxiv");

  // fundam.cls L42: `\newcommand{\runninghead}[2]{...}` — running-head
  // setter; preserved as toctitle / shortauthor frontmatter notes.
  // Witness 2307.02180.
  DefMacro!("\\runninghead{}{}",
    "\\@add@frontmatter{ltx:note}[role=runninghead]{#1: #2}");
  // fundam.cls L271: `\def\address#1{...\footnotetext{Address for ...}}`.
  // Surface as a note (ltx:contact would require a creator wrapper).
  DefMacro!("\\address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}");

  // fundam.cls L146-155 pre-declares standard theorem-like envs via
  // `\newtheorem{X}[definition]{X-display-name}`. Raw cls runs `\theorem
  // package` machinery that fails earlier, leaving these undefined.
  // Defensively register them via raw-TeX so amsmath/amsthm-style
  // `\begin{theorem}` / `\begin{lemma}` / etc. work. Witness 2307.02180,
  // 2308.08842.
  RawTeX!(r"\newtheorem{definition}{Definition}[section]");
  RawTeX!(r"\newtheorem{theorem}[definition]{Theorem}");
  RawTeX!(r"\newtheorem{fact}[definition]{Fact}");
  RawTeX!(r"\newtheorem{lemma}[definition]{Lemma}");
  RawTeX!(r"\newtheorem{example}[definition]{Example}");
  RawTeX!(r"\newtheorem{assumption}[definition]{Assumption}");
  RawTeX!(r"\newtheorem{proposition}[definition]{Proposition}");
  RawTeX!(r"\newtheorem{remark}[definition]{Remark}");
  RawTeX!(r"\newtheorem{corollary}[definition]{Corollary}");
  RawTeX!(r"\newtheorem{claim}[definition]{Claim}");
});
