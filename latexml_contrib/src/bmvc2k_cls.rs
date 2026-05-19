//! Stub for bmvc2k.cls (BMVC British Machine Vision Conference).
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");

  // bmvc2k frontmatter (L167+) — preserve author content.
  def_macro_noop("\\bmvaOneDot")?;
  DefMacro!("\\bmvaHangBox{}", "#1");
  // \addauthor{name}{email}{institution-id} — emit name as author,
  // email as ltx:note for preservation.
  DefMacro!("\\addauthor{}{}{}",
    "\\author{#1}\\@add@frontmatter{ltx:note}[role=email]{#2}");
  DefMacro!("\\addinstitution{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
});
