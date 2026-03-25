use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: undertilde.sty.ltxml
  // Obsolete package, but...
  DefMath!("\\utilde{}", "\u{007E}", operator_role => "UNDERACCENT");
});
