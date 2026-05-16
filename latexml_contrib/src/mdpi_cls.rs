//! Stub for MDPI journal class (Definitions/mdpi.cls, bundled by users).
//!
//! Real mdpi.cls L20-50 loads article + many packages including hyperref,
//! url, booktabs, ragged2e (for \justify), cleveref. Mirror those so
//! papers using \href, \hypersetup, \url, \justify, \crefrangelabelformat
//! don't error out. Witness 2410.21443.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("article");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("url");
  RequirePackage!("booktabs");
  RequirePackage!("ragged2e");
  RequirePackage!("cleveref");
  RequirePackage!("etoolbox");
  RequirePackage!("lineno");

  // MDPI frontmatter — gobble cleanly.
  DefMacro!("\\corresref[]{}", "");
  DefMacro!("\\externalbibliography{}", "");
  DefMacro!("\\firstpage{}", "");
  DefMacro!("\\firstpagenote{}", "");
  DefMacro!("\\corres[]{}", "");
  DefMacro!("\\Journal{}", "");
  DefMacro!("\\firstnote{}", "");
  DefMacro!("\\Address{}", "");
  DefMacro!("\\AuthorNames{}", "");
  DefMacro!("\\AuthorCitation{}", "");
  DefMacro!("\\dates{}{}{}", "");
  DefMacro!("\\authorinitials{}", "");
  // Additional MDPI frontmatter macros (mdpi.cls L530+). Witness 2412.13512.
  DefMacro!("\\Title{}", "\\title{#1}");
  DefMacro!("\\TitleCitation{}", "");
  DefMacro!("\\pubvolume{}", "");
  DefMacro!("\\pubyear{}", "");
  DefMacro!("\\issuenum{}", "");
  DefMacro!("\\reftitle{}", "");
  DefMacro!("\\PublishersNote", "");
  DefMacro!("\\articlenumber{}", "");
  DefMacro!("\\copyrightyear{}", "");
  DefMacro!("\\histreceived{}", "");
  DefMacro!("\\histrevised{}", "");
  DefMacro!("\\histaccepted{}", "");
  DefMacro!("\\historypublished{}", "");
  DefMacro!("\\SetCaptionDefault", "");
});
