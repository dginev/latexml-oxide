//! Stub for newpxmath.sty (Palatino math fonts).
use latexml_package::prelude::*;

LoadDefinitions!({
  // Map newpxmath variant font macros to their standard equivalents.
  Let!("\\varmathbb", "\\mathbb");
  Let!("\\vmathbb", "\\mathbb");
  Let!("\\vvmathbb", "\\mathbb");
  Let!("\\vvarmathbb", "\\mathbb");
});
