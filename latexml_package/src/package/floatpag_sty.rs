use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: floatpag.sty.ltxml
  // I don't suppose we really need to float any pages?
  DefMacro!("\\floatpagestyle{}", None);
  DefMacro!("\\rotfloatpagestyle{}", None);
  DefMacro!("\\thisfloatpagestyle{}", None);
});
