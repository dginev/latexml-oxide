//! orcidlink.sty — ORCID link support
//! Perl: orcidlink.sty.ltxml (52 lines)
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("hyperref");
  // Perl #2681: orcidlink.sty depends on tikz (even though we hardcode the SVG logo)
  RequirePackage!("tikz");

  // The link wrapper (`\lx@orcidlink`) and the iD logo (`\lx@orcidlogo`) are the
  // shared kernel asset (base_utilities.rs), reused by the frontmatter
  // `\lx@add@orcid`; orcidlink.sty just exposes them under its own names so there
  // is a SINGLE definition of each. \orcidlogo -> \lx@orcidlogo.
  Let!("\\orcidlogo", "\\lx@orcidlogo");

  // Perl orcidlink.sty.ltxml L29 passes `robust => 1` so \orcidlinkX
  // survives \write/\edef contexts (e.g. being rendered inside PDF
  // metadata or saved footnote text). Rust was missing the flag.
  // The emptiness tests stand for orcidlink.sty's `\ifstrempty` (etoolbox). They
  // are `\if\relax\detokenize{#1}\relax`, not Perl's `\ifx&#1&`
  // (orcidlink.sty.ltxml:28): a bare `&` is a column end when read at alignment
  // brace level 0 (tex.web §342), so `\textbf{Name~\orcidlink{…}}` in a
  // `p{…}` cell, whose `\bgroup` does not raise that level, ended the cell early
  // and left its mode-switch open (arXiv 2605.21922, 7 errors).
  DefMacro!("\\orcidlinkX{}{}{}",
    "\\lx@orcidlink{#2}{\\if\\relax\\detokenize{#1}\\relax\\else#1\\,\\fi\\orcidlogo\\if\\relax\\detokenize{#3}\\relax\\else\\,#3\\fi}",
    robust => true);

  // Default, Full, Compact and Inline versions
  DefMacro!("\\orcidlink{}",    "\\orcidlinkX{}{#1}{}");
  DefMacro!("\\orcidlinkf{}",   "\\orcidlinkX{}{#1}{https://orcid.org/#1}");
  DefMacro!("\\orcidlinkc{}",   "\\orcidlinkX{}{#1}{#1}");
  DefMacro!("\\orcidlinki{}{}", "\\orcidlinkX{#1}{#2}{}");
});
