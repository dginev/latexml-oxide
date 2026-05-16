//! Stub for CEUR-WS ceurart.cls.
//!
//! ceurart.cls is built on top of scrartcl + expl3/xparse. The class
//! defines `\sep` via `\tex_def:D \sep{\unskip,}` inside an expl3 block,
//! which our raw-load can't reliably execute. Most user-facing
//! frontmatter macros (`\ead`, `\fnmark`, etc.) use `\NewDocumentCommand`
//! with expl3 bodies that don't fully unfurl either.
//!
//! Provide gobble stubs for the frontmatter helpers and a plain `\sep`
//! so author/affiliation/keyword lists render in document text.
//!
//! Witness: 2501.13802, 2501.14238, 2501.16855, 2501.17381, 2502.01404,
//! 2502.02753, 2502.06743 — all `Error:undefined:\sep`.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("hyperref");
  RequirePackage!("xcolor");
  RequirePackage!("graphicx");
  RequirePackage!("etoolbox");
  RequirePackage!("booktabs");
  RequirePackage!("makecell");
  RequirePackage!("multirow");
  RequirePackage!("array");
  RequirePackage!("xspace");
  RequirePackage!("calc");
  RequirePackage!("natbib");

  // The core separator — used in author/affiliation/keyword lists.
  DefMacro!("\\sep", ",");

  // Frontmatter helpers (CEUR-WS / Elsevier conventions).
  DefMacro!("\\tnotetext[]{}", "");
  DefMacro!("\\tnotemark[]", "");
  DefMacro!("\\tnoteref[]{}", "");
  DefMacro!("\\fnmark[]", "");
  DefMacro!("\\fnref[]{}", "");
  DefMacro!("\\fntext[]{}", "");
  DefMacro!("\\cortext[]{}", "");
  DefMacro!("\\cormark[]", "");
  DefMacro!("\\corref[]", "");
  DefMacro!("\\affiliation[]{}", "");
  DefMacro!("\\address[]{}[]", "");
  DefMacro!("\\ead[]{}", "");
  DefMacro!("\\eadsep", "");
  DefMacro!("\\eadauthor", "");
  DefMacro!("\\orcidauthor{}{}", "");
  DefMacro!("\\urlauthor{}{}", "");
  DefMacro!("\\emailauthor{}{}", "");
  DefMacro!("\\creditauthor{}{}", "");
  DefMacro!("\\printcredits", "");
  DefMacro!("\\printemails", "");
  DefMacro!("\\printurls", "");
  DefMacro!("\\printorcid", "");
  DefMacro!("\\printtnotes", "");
  DefMacro!("\\copyrightyear{}", "");

  // Subtitle just becomes a textual addition.
  DefMacro!("\\subtitle{}", "");

  // CEUR-WS conference metadata.
  DefMacro!("\\conference{}", "");
  DefMacro!("\\copyrightclause{}", "");
  DefMacro!("\\ceurConference[]{}{}{}{}", "");
  DefMacro!("\\ceurEditors{}", "");
  DefMacro!("\\ceurVolumeNr{}", "");
  DefMacro!("\\ceurAuthors{}", "");
  DefMacro!("\\ceurTitle{}", "");
  DefMacro!("\\ceurLabel{}", "");
  DefMacro!("\\ceurRef{}", "");
  DefMacro!("\\ceurpubyear{}", "");
  DefMacro!("\\ceurwsurl{}", "");
  DefMacro!("\\ceurvolnr{}", "");
});
