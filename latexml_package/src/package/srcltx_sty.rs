//! srcltx.sty — source specials for DVI
//! Perl: srcltx.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("ifthen");

  DefMacro!("\\Input{}",          "\\input{#1}");
  DefMacro!("\\MainFile",         "\\jobname");
  DefMacro!("\\WinEdt{}",         "");
  DefMacro!("\\srcIncludeHook{}", "");
  DefMacro!("\\srcInputHook{}",   "");
  DefConditional!("\\ifSRCOK");
});
