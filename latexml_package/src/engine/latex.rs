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

  // SYNC_STATUS D0: Perl's LoadFormat is mutually exclusive —
  // `bootstrap + dump + constructs` when the dump exists, else
  // `bootstrap + _base + constructs`. Our default still loads both
  // (the add-only policy makes it safe but wastes ~10 ms / ~5 MB).
  //
  // `LATEXML_MUTEX_BASE_DUMP=1` opts into the Perl-style split for
  // experiments (v3.e bisection). When set, `latex_base` is skipped
  // whenever the dump load succeeds. See docs/DUMP_FORMAT_PERL_ANALYSIS.md
  // for the prerequisites (v3.a-v3.d) this gate relies on.
  let mutex_enabled = std::env::var("LATEXML_MUTEX_BASE_DUMP")
    .ok()
    .filter(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    .is_some();

  let dump_loaded_ok = if mutex_enabled {
    match crate::engine::latex_dump::load_definitions() {
      Ok(()) => true,
      Err(e) => {
        log::warn!("latex_dump (mutex-mode): {}", e);
        false
      },
    }
  } else {
    false
  };

  if !dump_loaded_ok {
    InnerPool!(latex_base);
  }

  // Perl: LoadPool('latex_dump') — precompiled latex.ltx state (expl3, fonts, captions).
  // Uses add-only policy: definitions already set by bootstrap+base are not overwritten.
  // Skipped when mutex-mode already ran it above.
  if !mutex_enabled {
    if let Err(e) = crate::engine::latex_dump::load_definitions() {
      log::warn!("latex_dump: {}", e);
    }
  }

  // Perl: LoadPool('latex_constructs') — semantic definitions (constructors, environments).
  InnerPool!(latex_constructs);
});
