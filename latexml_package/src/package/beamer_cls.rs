//! beamer.cls — Minimal stubs for beamer presentation class
//! Perl: beamer.cls.ltxml (1364 lines)
//!
//! Provides enough definitions for the beamer test to pass without loading
//! the raw beamer.cls (which exceeds the 5M token limit). Full beamer
//! support requires porting the complete Perl binding.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Load article.cls as the base class (beamer builds on article).
  // Don't load raw beamer.cls — its expansion chains exceed the token limit.
  RequirePackage!("article");

  // Frame environment — the core beamer construct.
  // Absorbs optional overlay spec and optional title/subtitle args.
  // Perl: DefEnvironment('{frame}[][]', '<ltx:slide...>...</ltx:slide>');
  DefEnvironment!("{frame}[][]",
    "<ltx:subsection _noautoclose='1'>#body</ltx:subsection>");

  // Overlay specification commands — stub as no-ops
  DefMacro!("\\visible{}", "#1");
  DefMacro!("\\uncover{}", "#1");
  DefMacro!("\\invisible{}", "#1");
  DefMacro!("\\only{}", "#1");
  DefMacro!("\\onslide", "");
  DefMacro!("\\temporal{}{}{}", "#2");
  DefMacro!("\\pause", "");
  DefMacro!("\\alt{}{}", "#1");

  // Frame structure
  DefMacro!("\\frametitle OptionalMatch:<> []{}",
    "\\par\\textbf{#3}\\par");
  DefMacro!("\\framesubtitle OptionalMatch:<> {}", "");

  // Insert counters
  DefMacro!("\\insertframenumber", "");
  DefMacro!("\\insertslidenumber", "");
  DefMacro!("\\insertpagenumber", "");
  DefMacro!("\\insertoverlaynumber", "");

  // Overlay environments
  DefEnvironment!("{onlyenv}", "#body");
  DefEnvironment!("{altenv}{}{}{}{}", "#body");
  DefEnvironment!("{alertenv}", "#body");
  DefEnvironment!("{uncoverenv}", "#body");
  DefEnvironment!("{actionenv}", "#body");
});
