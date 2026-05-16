//! Stub for Elsevier cas-dc.cls / cas-sc.cls (CAS journals double-column).
//!
//! The cas-* classes load cas-common.sty which uses xparse/expl3
//! NewDocumentCommand to define many frontmatter helpers. Our raw load
//! may not invoke them; provide gobble stubs for the most common
//! frontmatter macros.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  // cas-dc.cls L63: \RequirePackage{booktabs,makecell,multirow,array,colortbl,dcolumn,stfloats}.
  RequirePackage!("booktabs");
  RequirePackage!("multirow");
  RequirePackage!("array");
  RequirePackage!("colortbl");
  RequirePackage!("makecell");

  // cas-common frontmatter — gobble cleanly.
  DefMacro!("\\tnotetext[]{}", "");
  DefMacro!("\\tnotemark[]", "");
  DefMacro!("\\tnoteref[]{}", "");
  DefMacro!("\\fnmark[]", "");
  DefMacro!("\\fnref[]{}", "");
  DefMacro!("\\fntext[]{}", "");
  DefMacro!("\\nonumnote{}", "");
  DefMacro!("\\nonumtnotetext{}", "");
  DefMacro!("\\cortext[]{}", "");
  DefMacro!("\\cormark[]", "");
  DefMacro!("\\corref[]", "");
  DefMacro!("\\affiliation[]{}", "");
  DefMacro!("\\ead[]{}", "");

  // \sep — author/affil separator that cas-common defines.
  DefMacro!("\\sep", ",");
});
