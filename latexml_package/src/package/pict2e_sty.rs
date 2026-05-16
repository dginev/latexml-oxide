//! pict2e.sty — extended LaTeX picture environment.
//!
//! pict2e enhances the standard LaTeX picture environment with arbitrary-
//! slope `\line` / `\vector`, smooth curves, and quadratic/cubic Bezier
//! handling. The package's driver-dispatch chain (p2e-pdftex.def, etc.)
//! is heavy and fails our raw-load with "No suitable driver specified".
//! LaTeX picture is rendered as XML/SVG in our pipeline regardless of
//! driver, so the extended picture commands work the same way without
//! pict2e's PDF-specific drawing code.
//!
//! Stub as no-op (skip raw load + driver detection). The standard
//! picture environment is fully handled by latex_constructs.rs.
//! Witness 2503.14673 (pict2e error blocking 1 paper conversion).
use crate::prelude::*;

LoadDefinitions!({
  // Intentionally empty: skip pict2e's raw load. LaTeX picture
  // environment is handled by latex_constructs.rs regardless of
  // driver, so the PDF-specific drawing primitives that pict2e
  // would override are irrelevant in XML output.
  //
  // Define a few defensive stubs for pict2e-specific user-facing
  // CSes that some packages probe.
  DefMacro!("\\OriginalPictureCmds",          "");
  DefMacro!("\\pIIe@vector@ltx",              "");
  DefMacro!("\\pIIe@vector@pst",              "");
});
