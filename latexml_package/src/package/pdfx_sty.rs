//! pdfx.sty — PDF/X compliance, PDF/A, color profile support.
//!
//! Large package with expl3 + driver-detection cascade that fails our
//! raw-load. Perl LaTeXML has no pdfx binding and skips via
//! INCLUDE_STYLES=false. We produce XML/HTML, not PDF — so PDF/X
//! compliance is moot. Stub as no-op.
//!
//! Witness 2407.02288, 2408.13245, and ~12 papers per stage with
//! cascading 50-100 errors each (pdfx@*, xmp@*, set*colorprofile,
//! \hypersetup, \Hy@DisableOption, \selectcolormodel, ...).
use crate::prelude::*;

LoadDefinitions!({
  // Intentionally empty: skip pdfx's expl3-heavy raw-load. The output
  // is XML/HTML; PDF/X color-profile compliance is irrelevant.
});
