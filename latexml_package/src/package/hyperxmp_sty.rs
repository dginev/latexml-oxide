use crate::prelude::*;

// The effects of hyperxmp are already built into the LaTeXML binding for hyperref.
LoadDefinitions!({
  RequirePackage!("ifthen");

  // arXiv-fork (hyperxmp.sty.ltxml): user-facing helpers for XMP metadata
  // values. \XMPLangAlt passes alternate-language entries (TODO upstream:
  // should affect only *following* entries); \xmpquote / \xmpcomma allow
  // commas inside comma-separated XMP lists (TODO: real list splitting).
  Let!("\\XMPLangAlt", "\\@gobbletwo");
  Let!("\\xmpquote", "\\relax");
  DefMacro!("\\xmpcomma", ",");
});
