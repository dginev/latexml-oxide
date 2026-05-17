//! Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.).
//!
//! These classes share frontmatter macros like `\catchline{vol}{no}{year}{pgs}{pgsE}`,
//! `\title{...}`, `\Author{name}`. Raw load of the cls may fail before
//! defining these (cls uses complex font/page-layout machinery that
//! doesn't transfer to HTML conversion). Stubs preserve substantive
//! content as frontmatter notes.
//!
//! Witness: 2306.12455 (ws-ijgmmp.cls), 2306.15982 (ws-ijmpd.cls).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // \catchline{vol}{issue}{year}{pageStart}{pageEnd} — bibliographic
  // running-header metadata. Preserve as frontmatter note.
  DefMacro!("\\catchline{}{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=catchline]{Vol. #1, No. #2 (#3) #4--#5}");

  // \Journal{...} {...}{...}{...} — citation format helper (4-arg per WS conv).
  // Render the args inline as a generic citation string.
  DefMacro!("\\Journal{}{}{}{}", "{#1} {\\bf #2}, #3 (#4)");

  // No-op layout helpers.
  DefMacro!("\\copyrightheading{}", "");
  DefMacro!("\\paperBack", "");
  DefMacro!("\\catchlinefont", "\\footnotesize");
});
