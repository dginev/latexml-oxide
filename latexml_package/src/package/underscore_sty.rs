use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: underscore.sty.ltxml
  // Don't really need to change \_, but do need to make _ work in text!
  DefMacro!(T_ACTIVE!('_'), None, "\\ifmmode\\sb\\else\\textunderscore\\fi");
  at_begin_document(TokenizeInternal!(r"\catcode`_\active"))?;
});
