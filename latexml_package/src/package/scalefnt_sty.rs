use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: scalefnt.sty.ltxml
  // DefPrimitive('\scalefont{}', sub { MergeFont(scale => ToString($scale)); });
  // Stub as DefMacro — MergeFont(scale) not yet available in macro context
  DefMacro!("\\scalefont{}", None);
});
