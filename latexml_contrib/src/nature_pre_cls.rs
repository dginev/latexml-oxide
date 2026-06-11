//! Stub for nature-pre.cls (Nature pre-print template).
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

  // Nature pre L67 \newenvironment{affiliations} — list of author
  // affiliations. Render body so the affiliation text reaches XML.
  DefMacro!(T_CS!("\\begin{affiliations}"), None, "");
  DefMacro!(T_CS!("\\end{affiliations}"), None, "");
  // nature_mod L117 \newenvironment{addendum} — Acknowledgements /
  // Supplementary Info section wrapper. Pass body through so author
  // content reaches XML. Witness 2402.00473.
  DefMacro!(T_CS!("\\begin{addendum}"), None, "");
  DefMacro!(T_CS!("\\end{addendum}"), None, "");
  // \addendumlabel — bold-label macro used by \item inside addendum.
  DefMacro!("\\addendumlabel{}", "\\textbf{#1}\\hspace{1em}");
  // Preserve author content.
  DefMacro!(
    "\\correspondingauthor[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}"
  );
  DefMacro!(
    "\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}"
  );
});
