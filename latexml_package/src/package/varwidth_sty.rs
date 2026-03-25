use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: varwidth.sty.ltxml
  // Don't bother with distinction between {varwidth} and {minipage}
  DefMacro!("\\varwidth", "\\minipage");
  DefMacro!("\\endvarwidth", "\\endminipage");
  Let!("\\narrowragged", "\\raggedright");
});
