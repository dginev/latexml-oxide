use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


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
  def_macro_noop("\\bookmark[]{}")?;
  def_macro_noop("\\bookmarkdefinestyle{}{}")?;
  def_macro_noop("\\bookmarkget{}")?;
  def_macro_noop("\\BookmarkAtEnd{}")?;
  def_macro_noop("\\pdfbookmark[]{}{}")?;
  def_macro_noop("\\subpdfbookmark{}{}")?;
  def_macro_noop("\\belowpdfbookmark{}{}")?;
  def_macro_noop("\\currentpdfbookmark{}{}")?;
});
