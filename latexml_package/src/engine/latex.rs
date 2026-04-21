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

  // Load order — always the same, whether the dump is present or not:
  //   bootstrap → _base → dump → _constructs
  //
  // `_base` always runs BEFORE the dump so its closure-backed defs
  // (which can't be serialized) are installed. The dump then adds
  // serializable state (~25k entries of post-raw-load kernel state)
  // via add-only policy — CSes already defined by `_base` are not
  // overwritten. `_constructs` runs last and defines closure-backed
  // CSes that must be available regardless of dump state.
  //
  // This differs from Perl's strictly-mutually-exclusive `LoadFormat`
  // (bootstrap+dump+constructs XOR bootstrap+base+constructs) because
  // in our Rust port `_base` runs in ~3-5 ms of compiled code; there
  // is no meaningful speed win from skipping it, and keeping it loaded
  // means closure-backed defs don't vanish.
  InnerPool!(latex_bootstrap);

  // SYNC_STATUS D0 (d.1): stage a snapshot right after bootstrap.
  // ini_tex::dump_format reads it so its diff captures "bootstrap →
  // fully-initialized kernel", matching Perl's DumpFile semantics.
  // Has no effect at normal runtime (just stored in a thread-local).
  latexml_core::state::stage_snapshot("bootstrap");

  InnerPool!(latex_base);

  // Dump load — add-only: CSes already defined by _base skipped.
  // Honours `LATEXML_NODUMP=1` (Perl-parity) to disable the dump path.
  if let Err(e) = crate::engine::latex_dump::load_definitions() {
    log::warn!("latex_dump: {}", e);
  }

  InnerPool!(latex_constructs);

  // Retry any PA/MPA let-aliases whose target wasn't defined at
  // dump-load time (they were queued rather than silently dropped).
  // Classic example: `\let\a=\@tabacckludge` — `\@tabacckludge`
  // itself is defined in latex_constructs (which loads after the
  // dump), so the alias has to wait until now.
  let (applied, skipped) = latexml_core::dump_reader::flush_deferred_aliases();
  if applied + skipped > 0 {
    log::info!(
      "[latex_dump] deferred aliases: {} applied, {} skipped",
      applied, skipped
    );
  }
});
