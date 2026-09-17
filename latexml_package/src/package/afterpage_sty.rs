use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Real afterpage.sty:83: \long\def\afterpage#1{\gdef\AP@{{#1\par}}...} — the
  // body is stored by `\gdef`, which halves doubled `##` parameter markers to
  // `#`, and typeset after the current page. A web document has no page:
  // define and run the body at once (Perl afterpage.sty.ltxml:20 is a
  // `DefRegister` no-op that DROPS the body — OXIDIZED_DESIGN_DIVERGENCES
  // #225). Only for a document that loads the package: an undefined
  // `\afterpage` errors in LaTeX and Perl alike (sesamath-doc-fr relies on a
  // missing sesamath-doc.sty and fails in pdflatex — SHARED, not autoloaded).
  DefMacro!("\\afterpage {}", "\\gdef\\lx@afterpage@body{#1}\\lx@afterpage@body");
});
