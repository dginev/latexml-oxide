use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: fontspec.sty.ltxml
  // Preliminary support for xelatex
  RequirePackage!("xunicode");

  // Most of this is probably ignorable... at least initially.
  // And when not ignorable, may need some font re-thinking...

  // General Font selection
  DefMacro!("\\fontspec[]{}", None);
  DefMacro!("\\setmainfont[]{}", None);
  DefMacro!("\\setsansfont[]{}", None);
  DefMacro!("\\setmonofont[]{}", None);
  DefMacro!("\\newfontfamily DefToken []{}", None);
  DefMacro!("\\newfontface DefToken []{}", None);

  DefMacro!("\\setmathrm[]{}", None);
  DefMacro!("\\setmathsf[]{}", None);
  DefMacro!("\\setmathtt[]{}", None);
  DefMacro!("\\setboldmathrm[]{}", None);

  DefMacro!("\\defaultfontfeatures[]{}", None);
  DefMacro!("\\addfontfeatures[]{}", None);
});
