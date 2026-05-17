//! Stub for fundam.cls (Fundamenta Informaticae journal class).
//!
//! fundam.cls (v3.0, 2020) extends article for the Fundamenta Informaticae
//! journal. The raw cls defines `\publyear`, `\papernumber`, `\volume`,
//! `\issue` as simple metadata setters, but its preamble runs theorem.sty
//! and other env-heavy packages that fail mid-load, leaving the metadata
//! macros undefined. Witness 2305.16882.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("article");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  RequirePackage!("fancyhdr");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Article-metadata setters — raw cls assigns to internal `\@publyear`
  // etc.; HTML rendering surfaces as named frontmatter notes.
  DefMacro!("\\publyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\papernumber{}",
    "\\@add@frontmatter{ltx:note}[role=papernumber]{#1}");
  DefMacro!("\\volume{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\issue{}",
    "\\@add@frontmatter{ltx:note}[role=issue]{#1}");

  // \finalVersionForARXIV — toggles a `\finalarxivtrue` switch in raw
  // cls; HTML rendering ignores layout switches.
  DefMacro!("\\finalVersionForARXIV", "");
  DefConditional!("\\iffinalarxiv");
});
