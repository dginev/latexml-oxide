//! aistats2026.sty — AISTATS 2026 conference style file.
//!
//! Identifies as a class-style file (`\documentclass{article}` +
//! `\usepackage{aistats2026}`). The raw .sty includes a measurement-
//! based `\PackageError{Document}{Running heading author exceeds size
//! limitations}` that fires when the rendered running head exceeds
//! page width (aistats2026.sty L289-295). Since we render to XML/HTML
//! rather than PDF, page-width measurement is irrelevant and this
//! error is moot in our paradigm. Pre-define `\runningauthor` to a
//! benign value and stub `\@runningauthor` after raw load so the
//! measurement loop short-circuits without firing PackageError.
//!
//! Witness: 47 papers across stages 15-20 fail with this exact error.
use latexml_package::prelude::*;

LoadDefinitions!({
  // Pre-set \runningauthor to a placeholder so aistats2026.sty's
  // `\ifx\undefined\@runningauthor` test sees it as already defined
  // and skips the auto-fill from \@author (which is what triggers
  // the over-width PackageError).
  RawTeX!(r"\gdef\runningauthor#1{\gdef\@runningauthor{#1}}");
  RawTeX!(r"\gdef\@runningauthor{}");

  // Load the actual style for everything else (frontmatter, fancyhead
  // setup, etc.). The runningauthor pre-init above prevents the
  // \PackageError firing.
  InputDefinitions!("aistats2026", noltxml => true,
    extension => Some(Cow::Borrowed("sty")));
});
