use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: slashed.sty.ltxml
  // Unlikely we can do anything with the spacing fine-tuning,
  // so we'll just ignore this...
  DefMacro!("\\declareslashed{}{}{}{}{}", None);
  DefMacro!("\\sla@{}{}{}{}{}", "\\lx@slashed{#5}");
  // Let the \not handler in TeX.pool take care of this....
  DefMacro!("\\lx@slashed{}", "\\not{#1}");
  Let!("\\slashed", "\\lx@slashed");
});
