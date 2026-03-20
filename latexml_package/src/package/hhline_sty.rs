use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: hhline.sty.ltxml — stub: treat hhline as \hline
  DefMacro!("\\hhline Semiverbatim", "\\hline");
});
