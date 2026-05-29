//! Stub for gretsi.cls (French GRETSI conference template).
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

  // GRETSI frontmatter (gretsi.cls L79+) — preserve author content.
  // \resume → French abstract; route to abstract env.
  DefMacro!("\\resume{}",
    "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\auteurs", "\\author");
  // \auteur{name}{affiliation-id}{address}{email} — emit name as
  // author + address as note.
  DefMacro!("\\auteur{}{}{}{}",
    "\\author{#1}\\@add@frontmatter{ltx:note}[role=address]{#3}\\@add@frontmatter{ltx:note}[role=email]{#4}");
  DefMacro!("\\affils{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
});
