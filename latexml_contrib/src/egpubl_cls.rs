//! Stub for egpubl.cls (Eurographics conference proceedings).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Eurographics frontmatter — gobble cleanly.
  DefMacro!("\\teaser{}", "");
  DefMacro!("\\orcid{}", "");
  DefMacro!("\\ccsdesc[]{}", "");
  DefMacro!("\\printccsdesc", "");
  DefMacro!("\\ConfYear{}", "");
  DefMacro!("\\ConfEditors{}", "");
  DefMacro!("\\ConfEditorStrg{}", "");
  DefMacro!("\\EducationEditors{}", "");
  DefMacro!("\\TutorialEditors{}", "");
  DefMacro!("\\STARPresEditors{}", "");
  DefMacro!("\\DCEditors{}", "");
  DefMacro!("\\ShortPresEditors{}", "");
  DefMacro!("\\PosterEditors{}", "");
  DefMacro!("\\EventNoEds{}", "");
  DefMacro!("\\biberVersion{}", "");
  DefMacro!("\\BibtexOrBiblatex{}", "");
  DefMacro!("\\PrintedOrElectronic{}", "");
  DefMacro!("\\electronicVersion", "");
  DefMacro!("\\pdfSubject{}", "");
  DefMacro!("\\j@volume{}", "");
  DefMacro!("\\j@issue{}", "");
  DefMacro!("\\p@EGyear{}", "");
  DefMacro!("\\EGyear{}", "");

  // {CCSXML} env — ACM-style XML metadata block; suppress with the
  // comment package's \excludecomment idiom (egpubl L816). The
  // simplest faithful behaviour: an env that swallows its body.
  DefEnvironment!("{CCSXML}", "");
});
