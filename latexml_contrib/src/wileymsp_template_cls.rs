//! Stub for WileyMSP-template.cls (Wiley Mathematical Sciences Publishers).
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
  RequirePackage!("fancyhdr");
  RequirePackage!("ragged2e");
  // WileyMSP-template.cls L: `\RequirePackage{framed}` — needed for
  // {snugshade} environment used by template's editorial callout boxes.
  // Witness 2208.03623.
  RequirePackage!("framed");
  RequirePackage!("authblk");
  RequirePackage!("caption");
  // WileyMSP-template page-sets with geometry; papers use `\geometry{…}` in the
  // preamble without an explicit `\usepackage{geometry}` (witness 2306.02129).
  // Load it here (defining the no-op `\geometry`) instead of an engine-wide
  // kernel stub, which would make `\ifcsname geometry\endcsname` falsely true
  // for every document (broke 2005.03740). See latex_constructs.rs geometry note.
  RequirePackage!("geometry");

  DefMacro!(T_CS!("\\begin{affiliations}"), None, "");
  DefMacro!(T_CS!("\\end{affiliations}"), None, "");
  // Preserve author content as ltx:note frontmatter.
  DefMacro!(
    "\\correspondingauthor[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}"
  );
  DefMacro!(
    "\\corres{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
});
