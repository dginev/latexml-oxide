//! Stub for bmvc2k.cls (BMVC British Machine Vision Conference).
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
  RequirePackage!("graphicx");

  // bmvc2k frontmatter (L167+) — preserve author content.
  def_macro_noop("\\bmvaOneDot")?;
  DefMacro!("\\bmvaHangBox{}", "#1");
  // \addauthor{name}{email}{institution-id} — emit name as author,
  // email as ltx:note for preservation.
  DefMacro!(
    "\\addauthor{}{}{}",
    "\\author{#1}\\@add@frontmatter{ltx:note}[role=email]{#2}"
  );
  DefMacro!(
    "\\addinstitution{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}"
  );
});
