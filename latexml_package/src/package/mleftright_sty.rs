use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: mleftright.sty.ltxml
  // Since LaTeXML doesn't actually understand math inner atoms,
  // it doesn't introduce the "spurious" spacing that this package fixes.
  Let!("\\mleft",  "\\left");
  Let!("\\mright", "\\right");
  DefMacro!("\\mleftright",        None);
  DefMacro!("\\mleftrightrestore", None);
});
