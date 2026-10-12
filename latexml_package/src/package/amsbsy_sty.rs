//**********************************************************************
// See amsldoc
//**********************************************************************
use crate::{
  package::bm_sty::{BoldArgument, bold_math, define_bold_delimiter},
  prelude::*,
};
LoadDefinitions!({
  RequirePackage!("amsgen");

  // Perl amsbsy.sty.ltxml:23, the bounded constructor under a name of its own, reverting as `\boldsymbol`. amsbsy.sty:28-37
  // `\DeclareRobustCommand{\boldsymbol}` sets its argument in a box per math style, so a `\left` without its `\right`
  // there (`\boldsymbol{\left\langle} x \boldsymbol{\right\rangle}`) is TeX's error at the box's end: since 65d
  // `\left` opens a group the bounded argument would close (OXIDIZED_DESIGN #530), so it is set at the outer level,
  // as bm.sty's `\bm` sets it, with a Warning (bm_sty.rs `bold_math`, OXIDIZED_DESIGN #544).
  DefConstructor!("\\lx@boldsymbol@bold{}", "#1", bounded => true, font => { forcebold => true },
    alias => "\\boldsymbol");
  DefMacro!("\\boldsymbol{}", sub[(arg)] {
    Ok(bold_math("\\boldsymbol", T_CS!("\\lx@boldsymbol@bold"), arg, BoldArgument::Grouped))
  }, protected => true);
  define_bold_delimiter()?;

  // I think the intent is that you use \pmb to get bold, but your current font doesn't supply
  // a bold for those glyphs.  Since we're just forcing the glyph to be bold,
  // \pmb is probably \boldsymbol. or... ??
  Let!("\\pmb", "\\boldsymbol");
});
