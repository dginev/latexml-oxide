use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: auxhook.sty.ltxml
  DefPrimitive!("\\AddLineBeginAux{}", None);
  DefPrimitive!("\\AddLineBeginMainAux{}", None);
  DefPrimitive!("\\AddLineBeginPartAux{}", None);
});
