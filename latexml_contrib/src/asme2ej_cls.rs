//! Stub for asme2ej.cls (ASME journal class).
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

  // ASME-specific frontmatter — preserve author content.
  DefMacro!("\\setauthorname{}",
    "\\@add@frontmatter{ltx:note}[role=authorname]{#1}");
  DefMacro!("\\manuscriptnotenumber{}",
    "\\@add@frontmatter{ltx:note}[role=manuscriptno]{#1}");
  DefMacro!("\\confname{}",
    "\\@add@frontmatter{ltx:note}[role=conference]{#1}");
  DefMacro!("\\confyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
});
