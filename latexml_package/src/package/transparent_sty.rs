use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: transparent.sty.ltxml
  // \transparent{value} — set font opacity (stubbed: font opacity not yet supported)
  DefMacro!("\\transparent{}", None);
  DefMacro!("\\texttransparent{}{}", "{#2}");
});
