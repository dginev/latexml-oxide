///**********************************************************************
/// Organized following
///  "`LaTeX`: A Document Preparation System"
///   by Leslie Lamport
///   2nd edition
/// Addison Wesley, 1994
/// Appendix C. Reference Manual
///**********************************************************************
/// NOTE: This will be loaded after `TeX.pool`, so it inherits.
///**********************************************************************
use crate::prelude::*;

LoadDefinitions!({
  //**********************************************************************
  // Organized following
  //  "LaTeX: A Document Preparation System"
  //   by Leslie Lamport
  //   2nd edition
  // Addison Wesley, 1994
  // Appendix C. Reference Manual
  //**********************************************************************
  // NOTE: This will be loaded after TeX.pool.ltxml, so it inherits.
  //**********************************************************************

  // Perl: LaTeX.pool.ltxml — LoadPool('TeX'); LoadFormat('latex');
  LoadPool!("TeX");

  // Perl: LoadFormat('latex') does:
  //   LoadPool('latex_bootstrap') → LoadPool('latex_base') →
  //   LoadPool('latex_dump') → LoadPool('latex_constructs')
  // Match this order exactly:
  InnerPool!(latex_bootstrap);

  // SYNC_STATUS D0 (d.1): stage a snapshot right after bootstrap.
  // ini_tex::dump_format reads it so its diff captures "bootstrap →
  // fully-initialized kernel", matching Perl's DumpFile semantics.
  // Has no effect at normal runtime (the snapshot is just stored in a
  // thread-local; `_base` loading proceeds unchanged below).
  latexml_core::state::stage_snapshot("bootstrap");

  InnerPool!(latex_base);

  // Perl: LoadPool('latex_dump') — precompiled latex.ltx state (expl3, fonts, captions).
  // Uses add-only policy: definitions already set by bootstrap+base are not overwritten.
  if let Err(e) = crate::engine::latex_dump::load_definitions() {
    log::warn!("latex_dump: {}", e);
  }

  // Perl: LoadPool('latex_constructs') — semantic definitions (constructors, environments).
  InnerPool!(latex_constructs);
});
