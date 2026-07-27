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
//!
//! The same file's `keywords` block (the "Index Terms" list) is a bare
//! `\def\keywords`/`\def\endkeywords` pair, so `\begin{keywords}` found no
//! environment at all — the single largest `undefined` cluster in the sandbox
//! corpora (94 papers in sandbox-arxiv-2605, 49 in 2606; witnesses 2605.00480,
//! 2605.00698, 2605.00721, 2605.01187). See the `\keywords` binding below.
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
  // `\twoauthors{names1}{affil1}{names2}{affil2}` (spconf.sty L183-190) — the
  // side-by-side two-author title block, typeset as two `tabular`s that
  // overwrite `\@name` and blank `\@address`. Feed the same author machinery
  // as `\name`: `\and` separates the two groups, and within a group the `\\`
  // line after the names is that group's affiliation (`\lx@add@authors`,
  // base_utilities.rs L950-960). Witnesses 2605.05692, 2605.18923, 2605.26747.
  DefMacro!(
    "\\twoauthors{}{}{}{}",
    "\\author{#1 \\\\ #2 \\and #3 \\\\ #4}"
  );
  // The "Index Terms" block (spconf.sty L211-214):
  //   \def\keywords{\vspace{.5em}{\bfseries\textit{Index Terms}---\,\relax}}
  //   \def\endkeywords{\par}
  // — a plain-TeX environment pair, not a `\newenvironment`, so
  // `\begin{keywords}` runs `\keywords` and `\end{keywords}` runs
  // `\endkeywords`. Mirror the pair rather than declaring a `DefEnvironment!`,
  // both because that is what the `.sty` does and because it keeps a document
  // that calls the two macros directly (without `\begin`/`\end`) working.
  //
  // spconf.sty says this section was "adapted from IEEEtrans", and IEEEtran.cls
  // L5286-5288 typesets it identically (`\textit{\IEEEkeywordsname}---`). Perl
  // LaTeXML binds that very construct in IEEEtran.cls.ltxml L147-148 as the
  // structured `ltx:keywords` frontmatter, carrying the label in `@name` and
  // normalizing the print-only `---` separator to `:~`. Follow that precedent
  // exactly: the label is metadata (the XSLT renders `@name` as the block's
  // `<h6 class="ltx_title_keywords">` title), not content of the keyword list.
  // Raw-loaded spconf leaves it as inline body text instead — OXIDIZED_DESIGN #82.
  DefMacro!("\\spconf@keywordsname", "Index Terms");
  DefMacro!(
    "\\keywords",
    "\\lx@begin@keywords[name={\\spconf@keywordsname:~}]"
  );
  DefMacro!("\\endkeywords", "\\lx@end@keywords");
  // spconf uppercases the title; keep LaTeX's `\title` semantics (no forced
  // uppercase — casing is presentational and belongs in CSS).
  DefMacro!("\\ninept", "");
  def_macro_noop("\\copyrightnotice{}")?;
  def_macro_noop("\\toappear{}")?;
});
