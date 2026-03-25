use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: crop.sty.ltxml
  DefPrimitive!("\\crop []", None);
  DefPrimitive!("\\cropdef [] DefToken DefToken DefToken DefToken {}", None);
});
