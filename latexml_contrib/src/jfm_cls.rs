//! Stub for jfm.cls (Journal of Fluid Mechanics).
//!
//! Author-bundled class commonly used by JFM submissions. Defines a few
//! frontmatter helpers that aren't in standard article (`\aff`, `\corresp`,
//! `\affiliation`). The cls itself isn't raw-loaded reliably; stub the
//! author-facing macros so submissions don't error on undefined CS.
//! Witness 2312.07468.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");
  RequirePackage!("natbib");

  // jfm.cls L?: \def\aff#1{\ignorespaces\textsuperscript{#1}} — affil
  // marker as superscript.
  DefMacro!("\\aff{}", "\\textsuperscript{#1}");
  // jfm.cls: \def\corresp#1{\unskip\thanks{#1}} — corresponding author
  // text routed via \thanks.
  DefMacro!("\\corresp{}", "\\thanks{#1}");
  // \affiliation{...} — institution list, preserve as ltx:note.
  DefMacro!("\\affiliation{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
  // \keywords{...} — JFM has its own keywords macro.
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:keywords}{#1}");
});
