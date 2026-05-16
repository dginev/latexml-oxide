//! Stub for achemso.cls (ACS chemistry journals).
//!
//! achemso.cls is an article-derivative for ACS journals. Provides
//! authorship/affiliation primitives (\affiliation, \alsoaffiliation,
//! \altaffiliation, \email, \phone, \fax). Gobble for now since we
//! don't render ACS-style title pages.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("natbib");

  // ACS authorship primitives.
  DefMacro!("\\affiliation[]{}", "");
  DefMacro!("\\alsoaffiliation[]{}", "");
  DefMacro!("\\altaffiliation[]{}", "");
  DefMacro!("\\email{}", "");
  DefMacro!("\\phone{}", "");
  DefMacro!("\\fax{}", "");
  DefMacro!("\\suppinfo{}", "");
  DefMacro!("\\manuscript{}", "");
  DefMacro!("\\abbreviations{}", "");
  DefMacro!("\\acsAuthorList{}", "");
  DefMacro!("\\notetext{}", "");
  DefMacro!("\\acsSection{}", "");

  // {tocentry} environment — table of contents image, suppress.
  DefMacro!(T_CS!("\\begin{tocentry}"), None, "\\iffalse");
  DefMacro!(T_CS!("\\end{tocentry}"), None, "\\fi");

  // {acknowledgement} — ACS-spelt acknowledgement section.
  DefEnvironment!(
    "{acknowledgement}",
    "<ltx:acknowledgements>#body</ltx:acknowledgements>"
  );
});
