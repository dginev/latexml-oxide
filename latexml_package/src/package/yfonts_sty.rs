use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: yfonts.sty.ltxml — Not much to do for LaTeXML.

  DefPrimitive!("\\frakfamily", None, font => {family => "fraktur"});
  // These font families are otherwise unrecognized...
  DefPrimitive!("\\swabfamily", None, font => {family => "schwabacher"});
  DefPrimitive!("\\gothfamily", None, font => {family => "gothic"});

  // SHOULD set up fancy initials...
  DefMacro!("\\initfamily", None);
  DefPrimitive!("\\fraklines", None);

  DefMacro!("\\yinipar{}", "\\par\\noindent\\yinitpar{#1}");
  // SHOULD set the initial in fancy font.
  DefMacro!("\\yinitpar{}", "#1");

  // Nothing likely to ever be used, but for completeness...
  DefMacro!("\\gothdefault", "ygoth");
  DefMacro!("\\swabdefault", "yswab");
  DefMacro!("\\frakdefault", "yfrak");
  DefMacro!("\\initdefault", "yinitas");

  Let!("\\grq", "\\textquoteleft");
  Let!("\\grqq", "\\textquotedblleft");
});
