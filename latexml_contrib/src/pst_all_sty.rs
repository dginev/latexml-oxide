//! pst-all.sty — convenience loader for the full pstricks suite.
//!
//! pst-all chain-loads pst-tree / pst-coil / pst-text / pst-3d /
//! pst-eps / pst-fill / pstricks-add. Raw-loading these hits
//! PostScript-specific primitives we don't model (`\psk@rot`,
//! PSTricks parameter cascade) and floods the 100-error cap.
//!
//! Witness arXiv:1402.6510 — loads pst-all but never invokes a
//! single PSTricks command; Perl converts with 367 warnings.
//!
//! Match Perl: stub as a no-op so the raw-load is skipped.
//! pst-plot already has this treatment (see pst_plot_sty.rs).
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "pst-all.sty",
    "pst-all.sty is not implemented; loading pstricks core only."
  );
  // pst-all loads pst-tree/pst-coil/pst-text/pst-3d/pst-eps/pst-fill/
  // pstricks-add. We provide pstricks core so the common `\rput`,
  // `\cnode`, `\psset` etc. macros resolve. The fancier sub-packages
  // remain unstubbed for now (papers that use them get undefined-CS
  // warnings but not fatals).
  RequirePackage!("pstricks");
  // pst-all.sty L23 does `\RequirePackage{pst-node}`. Load our pst-node
  // binding (a clean stub — NOT the raw .sty) so node macros
  // (`\Cnode`/`\cnode`/`\rnode`/nc-*/…) resolve with correct
  // `(coord)`-consuming signatures. Without it, `\Cnode(1,1){000}` →
  // `undefined:\Cnode` (witness 1509.04932/1604.02906/1809.03593/…).
  RequirePackage!("pst-node");
  // A few connection macros below are kept as belt-and-suspenders for
  // the rare paper that loads pst-all but somehow not pst-node; pst-node
  // (loaded above) already defines these, so these are redundant
  // overrides with the same no-op behavior.
  DefMacro!("\\ncline OptionalMatch:* [] {} {}", "");
  DefMacro!("\\nccurve OptionalMatch:* [] {} {}", "");
  DefMacro!("\\ncarc OptionalMatch:* [] {} {}", "");
  DefMacro!("\\ncarcbox OptionalMatch:* [] {} {} {} {} {}", "");
  DefMacro!("\\pnode OptionalMatch:* () {}", "");
  DefMacro!("\\nput OptionalMatch:* {{}} {} {}", "");
  DefMacro!("\\aput OptionalMatch:* {{}} [] {}", "#3");
  DefMacro!("\\bput OptionalMatch:* {{}} [] {}", "#3");
  DefMacro!("\\ncangle OptionalMatch:* [] {} {}", "");
  DefMacro!("\\ncangles OptionalMatch:* [] {} {}", "");
  DefMacro!("\\ncloop OptionalMatch:* [] {} {}", "");
  DefMacro!("\\ncdiag OptionalMatch:* [] {} {}", "");
  DefMacro!("\\ncbar OptionalMatch:* [] {} {}", "");
  DefMacro!("\\ncput OptionalMatch:* {{}} [] {}", "#3");
  DefMacro!("\\cnodeput OptionalMatch:* [] () {} {}", "#4");
});
