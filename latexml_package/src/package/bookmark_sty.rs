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
  def_macro_noop("\\bookmarksetup{}")?;
  // bookmark.sty L134: `\newcommand*{\bookmarksetupnext}[1]{...}` — sets
  // options for the NEXT bookmark only. Cosmetic PDF-outline metadata with
  // no HTML analogue, so a no-op matches the sibling `\bookmarksetup` stub
  // (and Perl's effective behavior — Perl raw-loads bookmark.sty, defining
  // it, but its effect is PDF-only). Was the one public macro missing from
  // this stub. Witness 1707.07002 (Perl rc=0; Rust errored undefined).
  def_macro_noop("\\bookmarksetupnext{}")?;
  def_macro_noop("\\bookmark[]{}")?;
  def_macro_noop("\\bookmarkdefinestyle{}{}")?;
  def_macro_noop("\\bookmarkget{}")?;
  def_macro_noop("\\BookmarkAtEnd{}")?;
  def_macro_noop("\\pdfbookmark[]{}{}")?;
  def_macro_noop("\\subpdfbookmark{}{}")?;
  def_macro_noop("\\belowpdfbookmark{}{}")?;
  def_macro_noop("\\currentpdfbookmark{}{}")?;
});
