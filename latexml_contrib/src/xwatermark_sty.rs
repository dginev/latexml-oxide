//! Stub for xwatermark.sty — page-watermark package (visual-only).
//!
//! Watermarks have no XML/HTML output. Raw-loading xwatermark triggers
//! a transitive load of catoptions, which uses expl3-style macros our
//! engine doesn't fully emulate. The result is a 100-error cascade
//! that often OOMs (SIGKILL). Stub the public macros instead.
//!
//! Witness: 2202.02001, 2201.03132, 2202.12029, 2204.05441, 2204.12966
//! and ~20 sibling FATAL_137 (OOM-during-cascade) papers.
use latexml_package::prelude::*;

LoadDefinitions!({
  // No-op the public watermark API
  def_macro_noop("\\newwatermark{}")?;
  def_macro_noop("\\xnewwatermark[]{}")?;
  def_macro_noop("\\xnewwatermark*[]{}")?;
  def_macro_noop("\\removewatermark{}")?;
  def_macro_noop("\\DeclareWatermarkParser{}")?;
  // Options (printwatermark, draft, etc.) handled by DeclareOption no-ops
  for opt in ["printwatermark", "draft", "final", "noprintwatermark"].iter() {
    DeclareOption!(*opt, None);
  }
  DeclareOption!(None, {});
  ProcessOptions!();
});
