//! Stub for ecai.cls (ECAI conference class).
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");

  // ECAI frontmatter (ecai.cls L1290) — preserve paper ID as note.
  DefMacro!("\\paperid{}",
    "\\@add@frontmatter{ltx:note}[role=paperid]{#1}");
  def_macro_noop("\\makepaperid")?;
  // ECAI authors use \orcid for ORCID identifier; preserve as note.
  // Witness 2501.02040 + 3 ecai papers.
  DefMacro!("\\orcid{}",
    "\\@add@frontmatter{ltx:note}[role=orcid]{#1}");
  // {ack} environment — acknowledgments block. Emit as structural
  // ltx:acknowledgements (vs flattening into a generic section).
  // Witness 2408.16081.
  DefEnvironment!("{ack}", "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  // \ecaisubmission — page-numbering toggle for submission mode. No-op
  // (ecai.cls L1100-ish flips internal `\if@ecai@subm` then issues
  // `\pagenumbering{arabic}\setcounter{page}{1}`). The visible effect
  // is page numbers in print; in HTML the page concept is meaningless.
  // Witness 2305.13804.
  def_macro_noop("\\ecaisubmission")?;
});
