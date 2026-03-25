use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: stfloats.sty.ltxml
  // Not much to do here
  DefMacro!("\\fnbelowfloat", None);
  DefMacro!("\\fnunderfloat", None);
  DefMacro!("\\setbaselinefloat", None);
  DefMacro!("\\setbaselinefixed", None);
});
