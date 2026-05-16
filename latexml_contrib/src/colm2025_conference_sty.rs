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
  // colm2025_conference.sty L16-17 also declares \ifcolmpreprint.
  // Witnesses: 2504.03048, 2504.05625, 2504.09394 (papers passing
  // [preprint] class option which calls \colmpreprinttrue).
  DefConditional!("\\ifcolmpreprint");
});
