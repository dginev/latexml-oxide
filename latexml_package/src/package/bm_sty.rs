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

  // bm.sty L222-227: `\DeclareBoldMathCommand[<bold|heavy>]\<name>{<body>}`
  // declares `\name` as a bold-math wrapper around `body`. We collapse
  // both bold and heavy variants into our single `\bm` wrapper (we don't
  // distinguish bold vs heavy in the XML output anyway, mirroring the
  // \heavysymbol → \boldsymbol Let above). Surpass-Perl: Perl bm.sty.ltxml
  // doesn't carry `\DeclareBoldMathCommand` either, so this is a Rust-only
  // addition. Witness: arxiv-examples/1205.4484 (macros.tex defines
  // \boldlangle / \boldrangle / \boldlvert / \boldrvert via
  // \DeclareBoldMathCommand).
  DefMacro!("\\DeclareBoldMathCommand[]{}{}", "\\newcommand{#2}{\\bm{#3}}");
});
