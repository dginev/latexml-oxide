use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: bm.sty.ltxml
  // Since we're really punting the whole question of what fonts have
  // bold variants of which characters, this should be enough:
  DefConstructor!("\\bm{}", "#1", bounded => true, require_math => true, font => { forcebold => true });
  DefMacro!("\\bmdefine{}{}", "\\newcommand{#1}{\\bm{#2}}");
  Let!("\\boldsymbol", "\\bm");

  // Should we make a distinction between bold & heavy?
  Let!("\\hm",          "\\bm");
  Let!("\\heavysymbol", "\\boldsymbol");
  Let!("\\hmdefine",    "\\bmdefine");
  Let!("\\heavymath",   "\\boldmath");
});
