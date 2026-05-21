//! srcltx.sty — source specials for DVI
//! Perl: srcltx.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("ifthen");

  DefMacro!("\\Input{}",          "\\input{#1}");
  DefMacro!("\\MainFile",         "\\jobname");
  def_macro_noop("\\WinEdt{}")?;
  def_macro_noop("\\srcIncludeHook{}")?;
  def_macro_noop("\\srcInputHook{}")?;
  DefConditional!("\\ifSRCOK");
});
