//! spconf.sty / INTERSPEECH2021.sty — the ICASSP/older-Interspeech frontmatter
//! package (loaded on top of `\documentclass{article}`).
//!
//! Both define the single-argument conference convention
//!   `\name{Author1$^1$, Author2$^2$, …}`  `\address{$^1$Inst … $^2$ …}`
//!   `\email{…}`
//! (spconf.sty L170-172, INTERSPEECH2021.sty L171-173). With no binding the
//! raw `.sty` `\def\name#1{\gdef\@name{…}}` merely stashes the names and
//! article's structural `\maketitle` never emits them → zero creators (witness
//! 2309.14838, 2405.13379, 2605.10272). Route `\name` through the standard
//! author machinery so the comma/superscript-marked list becomes structured
//! creators, and keep `\address`/`\email` as frontmatter.
use latexml_package::prelude::*;

LoadDefinitions!({
  // `\name{names}` — the whole author list in one argument. Hand it to `\author`
  // (→ `\lx@add@authors`), which splits the comma / superscript-marked list into
  // individual creators and links their affiliation superscripts.
  DefMacro!("\\name{}", "\\author{#1}");
  // `\address{affils}` — the (superscript-numbered) affiliation block. Preserve
  // as a frontmatter note so the numbered institutions are kept.
  DefMacro!(
    "\\address{}",
    "\\lx@add@frontmatter{ltx:note}[role=address]{#1}"
  );
  DefMacro!(
    "\\email{}",
    "\\lx@add@frontmatter{ltx:note}[role=email]{#1}"
  );
  // spconf uppercases the title; keep LaTeX's `\title` semantics (no forced
  // uppercase — casing is presentational and belongs in CSS).
  DefMacro!("\\ninept", "");
  def_macro_noop("\\copyrightnotice{}")?;
  def_macro_noop("\\toappear{}")?;
});
