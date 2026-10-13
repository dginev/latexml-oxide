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

  // amsbsy.sty:38-39, 58-69: `\binrel@{X}` lets `\binrel@@` be `\mathbin`, `\mathrel` or `\relax` by the spacing a
  // box measures around X, and `\math@atom{X}{Y}` sets Y as X's class. No box measures math spacing here, so
  // `\binrel@@` passes Y through ungrouped: Y keeps the class its own content has (the operator a paper restyles is
  // still one), where an ordinary group would hide it. Papers call them for their own operators: the first error of
  // 161 run-340 papers (witnesses 2103.00817, 0709.1048, 1310.2330).
  RawTeX!(
    r"\def\binrel@#1{\def\binrel@@##1{##1}}
\let\binrel@@\relax
\def\math@atom#1#2{\binrel@{#1}\binrel@@{#2}}"
  );
});
