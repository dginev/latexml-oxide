use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: fontspec.sty.ltxml
  // Preliminary support for xelatex
  RequirePackage!("xunicode");

  // Most of this is probably ignorable... at least initially.
  // And when not ignorable, may need some font re-thinking...

  // General Font selection. fontspec v2 puts the feature list AFTER the
  // font name (`\setmonofont{DejaVu Sans Mono}[Scale=…]`) — the `[]{}[]`
  // signatures absorb both the v1 pre-optional and v2 post-optional forms
  // (hvfloat/libertinus-otf corpus preambles use the v2 form bare).
  def_macro_noop("\\fontspec[]{}[]")?;
  def_macro_noop("\\setmainfont[]{}[]")?;
  def_macro_noop("\\setsansfont[]{}[]")?;
  def_macro_noop("\\setmonofont[]{}[]")?;
  def_macro_noop("\\newfontfamily DefToken []{}[]")?;
  def_macro_noop("\\newfontface DefToken []{}[]")?;

  def_macro_noop("\\setmathrm[]{}")?;
  def_macro_noop("\\setmathsf[]{}")?;
  def_macro_noop("\\setmathtt[]{}")?;
  def_macro_noop("\\setboldmathrm[]{}")?;

  def_macro_noop("\\defaultfontfeatures[]{}")?;
  def_macro_noop("\\addfontfeatures[]{}")?;
  def_macro_noop("\\addfontfeature[]{}")?;
  def_macro_noop("\\IfFontExistsTF{}{}{}")?;
});
