use crate::prelude::*;
// placeins.sty — float barriers (no-op since floats stay where found)

LoadDefinitions!({
  DeclareOption!("section", None);
  DeclareOption!("above", None);
  DeclareOption!("below", None);
  DeclareOption!("verbose", None);
  // Perl placeins.sty.ltxml L21: `ProcessOptions()` consumes the
  // declared no-ops so user-side `\usepackage[section]{placeins}`
  // doesn't leave unprocessed options behind. Rust was missing this.
  ProcessOptions!();
  // placeins.sty:30: `\FloatBarrier` begins with `\par`, then flushes the held floats, which
  // stay where found here. Perl's (L24) is a no-op, so the paragraphs around it ran together
  // (KPE #369).
  DefMacro!("\\FloatBarrier", "\\par");
});
