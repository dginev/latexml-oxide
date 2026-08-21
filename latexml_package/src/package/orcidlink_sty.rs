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
  DefMacro!("\\orcidlinkX{}{}{}",
    "\\lx@orcidlink{#2}{\\ifx&#1&\\else#1\\,\\fi\\orcidlogo\\ifx&#3&\\else\\,#3\\fi}",
    robust => true);

  // Default, Full, Compact and Inline versions
  DefMacro!("\\orcidlink{}",    "\\orcidlinkX{}{#1}{}");
  DefMacro!("\\orcidlinkf{}",   "\\orcidlinkX{}{#1}{https://orcid.org/#1}");
  DefMacro!("\\orcidlinkc{}",   "\\orcidlinkX{}{#1}{#1}");
  DefMacro!("\\orcidlinki{}{}", "\\orcidlinkX{#1}{#2}{}");
});
