//! Stub for egpubl.cls (Eurographics conference proceedings).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Eurographics frontmatter — preserve author-supplied content into
  // ltx:note frontmatter entries with role markers.
  DefMacro!("\\teaser{}",
    "\\@add@frontmatter{ltx:note}[role=teaser]{#1}");
  DefMacro!("\\orcid{}",
    "\\@add@frontmatter{ltx:note}[role=orcid]{#1}");
  DefMacro!("\\ccsdesc[]{}",
    "\\@add@frontmatter{ltx:classification}[scheme=ccs]{#2}");
  DefMacro!("\\printccsdesc", "");
  DefMacro!("\\ConfYear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\ConfEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}");
  // Editor sub-roles — gobble the format-style strings (\ConfEditorStrg
  // formatter) but preserve actual editor lists.
  DefMacro!("\\ConfEditorStrg{}", "");
  DefMacro!("\\EducationEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-education]{#1}");
  DefMacro!("\\TutorialEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-tutorial]{#1}");
  DefMacro!("\\STARPresEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-star]{#1}");
  DefMacro!("\\DCEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-dc]{#1}");
  DefMacro!("\\ShortPresEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-short]{#1}");
  DefMacro!("\\PosterEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-poster]{#1}");
  DefMacro!("\\EventNoEds{}",
    "\\@add@frontmatter{ltx:note}[role=event-no-eds]{#1}");
  // Bibliography format selectors — internal, gobble.
  DefMacro!("\\biberVersion{}", "");
  DefMacro!("\\BibtexOrBiblatex{}", "");
  DefMacro!("\\PrintedOrElectronic{}", "");
  DefMacro!("\\electronicVersion", "");
  DefMacro!("\\pdfSubject{}",
    "\\@add@frontmatter{ltx:note}[role=subject]{#1}");
  // Internal counters — gobble (won't be user-content).
  DefMacro!("\\j@volume{}", "");
  DefMacro!("\\j@issue{}", "");
  DefMacro!("\\p@EGyear{}", "");
  DefMacro!("\\EGyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");

  // {CCSXML} env — ACM-style XML metadata block; suppress with the
  // comment package's \excludecomment idiom (egpubl L816). The
  // simplest faithful behaviour: an env that swallows its body.
  DefEnvironment!("{CCSXML}", "");
});
