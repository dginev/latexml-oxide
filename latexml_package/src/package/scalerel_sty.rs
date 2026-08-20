use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // arXiv/html_feedback#6895: the `scalerel` package has no `.ltxml` binding in
  // Perl OR Rust, and the raw `.sty` load leaves `\scalerel` undefined — so an
  // inline icon built with `\scalerel*` (the `\orcidicon` of arXiv:2608.12272)
  // raised `Error:undefined:\scalerel` and rendered its picture unscaled ("too
  // big / multi-line"). Beyond Perl 0.8.8, which errors identically.
  //
  // `\scalerel*[maxwidth]{obj}{ref}` scales `obj` to the height of `ref`, aspect
  // preserved (scalerel.sty L68-84). The dominant use is an inline object scaled
  // to the surrounding *text* height, so — box-measurement scaling being
  // unavailable in this engine — we wrap `obj` in an inline-block that CSS sizes
  // to text height (`.ltx_scalerel`, `LaTeXML.css`). The `[maxwidth]` cap
  // (default `99in`, i.e. unbounded) is accepted and dropped. The starred form
  // yields just the scaled object; the plain `\scalerel{obj}{ref}` appends `ref`
  // afterwards (scalerel.sty L84, `\scalerelplus`).
  DefMacro!("\\scalerel", "\\@ifstar\\lx@scalerel@star\\lx@scalerel@plus");
  DefMacro!("\\lx@scalerel@star []{}{}", "\\lx@scalerel@obj{#2}");
  DefMacro!("\\lx@scalerel@plus []{}{}", "\\lx@scalerel@obj{#2}#3");
  DefConstructor!("\\lx@scalerel@obj{}",
    "<ltx:inline-block class='ltx_scalerel'>#1</ltx:inline-block>",
    mode => "restricted_horizontal", enter_horizontal => true, bounded => true);
  // `\stretchrel` stretches ignoring the aspect ratio; aspect-preserving is the
  // safe default for an inline icon, so alias it to `\scalerel` (scalerel.sty L86).
  Let!("\\stretchrel", "\\scalerel");
});
