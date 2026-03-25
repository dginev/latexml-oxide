use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ellipsis.sty.ltxml
  DefMacro!("\\ellipsisgap", None);
  DefMacro!("\\ellipsispunctuation", ",.:;!?");
  // \textellipsis already defined.
  DefMacro!("\\midwordellipsis", "\\textellipsis");
});
