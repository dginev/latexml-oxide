//! Stub for oup-authoring-template.cls (Oxford University Press journal template).
//!
//! User-bundled OUP class with many metadata setters and font/layout
//! configuration. Provide content-preserving stubs for the user-facing
//! metadata so author content reaches the XML output.
//!
//! Witness: 2503.21884 (\\journaltitle, \\DOI, \\access, ...).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");
  RequirePackage!("array");
  RequirePackage!("caption");

  // OUP-specific metadata setters — preserve content.
  DefMacro!("\\journaltitle{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\DOI{}",
    "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\access{}",
    "\\@add@frontmatter{ltx:note}[role=access]{#1}");
  DefMacro!("\\copyrightyear{}",
    "\\@add@frontmatter{ltx:note}[role=copyright-year]{#1}");
  DefMacro!("\\copyrightstatement{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}");
  DefMacro!("\\pubyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\appnotes{}",
    "\\@add@frontmatter{ltx:note}[role=appnotes]{#1}");
  DefMacro!("\\authormark{}", "\\textsuperscript{#1}");
  DefMacro!("\\corresp{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\firstpage{}",
    "\\@add@frontmatter{ltx:note}[role=firstpage]{#1}");
  DefMacro!("\\titlemark{}",
    "\\@add@frontmatter{ltx:note}[role=titlemark]{#1}");
  DefMacro!("\\runninghead{}",
    "\\@add@frontmatter{ltx:note}[role=runninghead]{#1}");
  DefMacro!("\\received{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}");
  DefMacro!("\\revised{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1}");
  DefMacro!("\\accepted{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1}");
  DefMacro!("\\editor{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}");
  DefMacro!("\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}");
  // SetCrop/SetTrim/SetBleed are layout configs — gobble.
  DefMacro!("\\SetCrop{}{}", "");
  DefMacro!("\\SetTrim{}{}", "");
  DefMacro!("\\SetBleed{}{}", "");
});
