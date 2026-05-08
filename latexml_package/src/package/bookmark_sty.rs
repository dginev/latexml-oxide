use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // bookmark.sty stub. Perl previously raw-loaded the package, but its
  // driver-file dispatch (`\InputIfFileExists{bkm-\BKM@driver.def}`)
  // expands `\BKM@driver` (=`dvips`) to load `bkm-dvips.def`, which
  // brings in `\AddToHook{shipout/lastpage}{...}` + complex
  // `\BKM@entry` machinery that hits Rust's 100M token-limit at load
  // time (drivers: 2310.15090, 2203.01231 — both Perl-shared in the
  // sense that Perl also blows past `pushbacklimit=599999` from
  // ar5iv preset).
  //
  // bookmark generates PDF outline metadata; in HTML conversion this
  // is decorative — stub all user-facing macros so paper-level
  // `\bookmarksetup{...}` etc. become no-ops, and skip the raw-load
  // entirely. Same minimal-stub approach as `xkeyval` for compat
  // packages whose features have no HTML analogue.

  DeclareOption!(None, {});
  ProcessOptions!();

  RequirePackage!("hyperref");

  // Public macros — all become no-ops.
  DefMacro!("\\bookmarksetup{}", "");
  DefMacro!("\\bookmark[]{}", "");
  DefMacro!("\\bookmarkdefinestyle{}{}", "");
  DefMacro!("\\bookmarkget{}", "");
  DefMacro!("\\BookmarkAtEnd{}", "");
  DefMacro!("\\pdfbookmark[]{}{}", "");
  DefMacro!("\\subpdfbookmark{}{}", "");
  DefMacro!("\\belowpdfbookmark{}{}", "");
  DefMacro!("\\currentpdfbookmark{}{}", "");
});
