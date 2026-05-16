//! Stub for SCIS2024.cls (Science China Information Sciences 2024).
//!
//! Defines a large set of frontmatter helpers (\ArticleType, \DOI,
//! \Year, \Month, etc.) via `\let\@X\@empty \def\X#1{\def\@X{#1}}`.
//! Our raw-load currently routes to OmniBus instead of the in-archive
//! .cls, leaving every \X CS undefined.
//!
//! Witness: 2503.01116 (14 frontmatter undefined cascade).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amsfonts");
  RequirePackage!("amssymb");
  RequirePackage!("bm");
  RequirePackage!("multicol");
  RequirePackage!("mathrsfs");
  RequirePackage!("pifont");
  RequirePackage!("graphicx");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("booktabs");
  RequirePackage!("tabularx");
  RequirePackage!("caption");
  RequirePackage!("subfig");
  RequirePackage!("cite");

  // SCIS2024 metadata setters — all gobble the arg (treated as
  // bibliographic info elsewhere; we don't render header).
  DefMacro!("\\ArticleType{}", "");
  DefMacro!("\\SpecialTopic{}", "");
  DefMacro!("\\Year{}", "");
  DefMacro!("\\Month{}", "");
  DefMacro!("\\Vol{}", "");
  DefMacro!("\\No{}", "");
  DefMacro!("\\AuthorMark{}", "");
  DefMacro!("\\AuthorCitation{}", "");
  DefMacro!("\\BeginPage{}", "");
  DefMacro!("\\EndPage{}", "");
  DefMacro!("\\DOI{}", "");
  DefMacro!("\\ArtNo{}", "");
  DefMacro!("\\ReceiveDate{}", "");
  DefMacro!("\\ReviseDate{}", "");
  DefMacro!("\\AcceptDate{}", "");
  DefMacro!("\\OnlineDate{}", "");
  DefMacro!("\\contributions{}", "");
  DefMacro!("\\luntan", "");
  DefMacro!("\\oa", "");
  DefMacro!("\\Acknowledgements", "\\section*{Acknowledgements}");
});
