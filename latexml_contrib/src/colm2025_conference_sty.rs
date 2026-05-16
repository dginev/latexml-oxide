//! Stub for colm2025_conference.sty (COLM 2025 conference template).
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("natbib");

  // Author-list separators (colm L107-153).
  DefMacro!("\\And", " ");
  DefMacro!("\\AND", " ");
  DefMacro!("\\Ands", " ");
  // ICLR/NeurIPS-style author email-aside.
  DefMacro!("\\affilmark{}", "");
  DefMacro!("\\thanksauthor", "");
  DefConditional!("\\ifcolmsubmission");
  DefConditional!("\\ifcolmfinal");
});
