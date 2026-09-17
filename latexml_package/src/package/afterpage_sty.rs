use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Real afterpage.sty:83: \long\def\afterpage#1{\gdef\AP@{{#1\par}}...}
  // In LaTeX, storing #1 in \gdef collapses doubled `##` parameter markers to `#`.
  // In a web converter without physical page breaks, define and invoke immediately.
  DefMacro!("\\afterpage {}", "\\gdef\\lx@afterpage@body{#1}\\lx@afterpage@body");
});
