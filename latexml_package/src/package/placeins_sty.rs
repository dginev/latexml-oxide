use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: placeins.sty.ltxml
  for option in ["section", "above", "below", "verbose"] {
    DeclareOption!(option, None);
  }
  ProcessOptions!();

  // Basically no-op, since floats stay where they're found.
  DefMacro!("\\FloatBarrier", None);
});
