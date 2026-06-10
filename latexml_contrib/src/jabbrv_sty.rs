//! Stub for jabbrv.sty (journal-abbreviation package).
//!
//! jabbrv provides `\JournalTitle{<full name>}` which abbreviates journal names via
//! big `jabbrv-ltwa-*.ldf` lists. It is shipped with papers (not in every TeX Live)
//! and pulled in by classes like wlscirep via `\RequirePackage{jabbrv}`. Two problems
//! with the real package under LaTeXML:
//!   (1) the OmniBus class-fallback dep-scan can't load a shipped-only raw `.sty`
//!       (notex gate) → `\JournalTitle` undefined (~42 CONVERR_1 papers), and
//!   (2) when jabbrv IS loaded via the class `\RequirePackage` path it leaves an
//!       `\emph` group imbalance (~95 errors — SHARED with Perl, which raw-loads it).
//! So the real raw-load is strictly WORSE than leaving it out (Perl = 95 errors;
//! Rust currently = 1). This stub provides jabbrv's public API cleanly, WITHOUT the
//! `.ldf` abbreviation machinery: `\JournalTitle` outputs the full journal name
//! (faithful enough for XML — the ISO abbreviation is a print convention), and the
//! `\Define*` declarations are no-ops. Result: `\JournalTitle` defined, 0 errors —
//! surpasses Perl's broken raw-load. Witness 2112.00489 (wlscirep).
use latexml_package::prelude::*;

LoadDefinitions!({
  DefMacro!("\\JournalTitle{}", "#1");
  DefMacro!("\\DefineJournalAbbreviation{}{}", "");
  DefMacro!("\\DefineJournalException{}{}", "");
  DefMacro!("\\DefineJournalPartialAbbreviation{}{}", "");
  DefMacro!("\\DefineSpuriousJournalWord{}", "");
});
