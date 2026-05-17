//! Stub for colm2025_conference.sty (COLM 2025 conference template).
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("natbib");
  // Some COLM 2025 templates author-edit the .sty to add `\definecolor`
  // calls before users `\usepackage{color}`. Eager-load color/xcolor so
  // the templates' early color definitions don't trip "\\definecolor
  // undefined". Witness 2503.21480 (definecolor at colm2025 L11).
  RequirePackage!("color");
  RequirePackage!("xcolor");

  // Author-list separators (colm L107-153).
  DefMacro!("\\And", " ");
  DefMacro!("\\AND", " ");
  DefMacro!("\\Ands", " ");
  // ICLR/NeurIPS-style author email-aside.
  // \affilmark{N,M,...} — affiliation superscript markers on the
  // author line. Author content; emit as superscript inline.
  DefMacro!("\\affilmark{}", "\\textsuperscript{#1}");
  DefMacro!("\\thanksauthor", "");
  DefConditional!("\\ifcolmsubmission");
  DefConditional!("\\ifcolmfinal");
  // colm2025_conference.sty L16-17 also declares \ifcolmpreprint.
  // Witnesses: 2504.03048, 2504.05625, 2504.09394 (papers passing
  // [preprint] class option which calls \colmpreprinttrue).
  DefConditional!("\\ifcolmpreprint");
});
