//! Stub for `extarticle.cls` (extsizes bundle — article with extended
//! font-size options like 8pt/9pt/14pt/17pt/20pt).
//!
//! Font sizes are layout-only; for XML/HTML output we don't care. Route
//! to plain `article` and accept the standard size set. Same pattern
//! for sibling classes extbook / extreport / extletter / extproc.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("article");
});
