//! Stub for wlpeerj.cls (Wiley PeerJ template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Frontmatter — preserve author content.
  // \corrauthor[mark]{name}{email} — emit name as author + email note.
  DefMacro!("\\corrauthor[]{}{}",
    "\\author{#2}\\@add@frontmatter{ltx:note}[role=email]{#3}");
  // \authoraffiliation[mark]{name}{affil} — emit name + affil note.
  DefMacro!("\\authoraffiliation[]{}{}",
    "\\author{#2}\\@add@frontmatter{ltx:note}[role=affiliation]{#3}");
  DefMacro!("\\affil[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
});
