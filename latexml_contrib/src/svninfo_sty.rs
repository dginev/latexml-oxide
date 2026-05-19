use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
/// Routes inline macro expansion (each ~960 B of .text) through one
/// runtime call. Engine bootstrap pays parse_prototype once per entry.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  RequirePackage!("ifthen");
  RequirePackage!("eso-pic");
  RequirePackage!("fancyheadings");
  // Perl-parity stubs: every \svn* CS is a `Tokens()` no-op in
  // ar5iv-bindings/svninfo.sty.ltxml L23-44 (matching upstream SVN
  // keyword placeholders that have no print-time equivalent in LaTeXML).
  def_macro_noop("\\svnInfo SkipSpaces Match:$ Until:$")?;
  def_macro_noop("\\svnKeyword SkipSpaces Match:$ Until:$")?;
  def_macro_noop("\\svnId")?;
  def_macro_noop("\\svnInfoDate")?;
  def_macro_noop("\\svnInfoDay")?;
  def_macro_noop("\\svnInfoFile")?;
  def_macro_noop("\\svnInfoHeadURL")?;
  def_macro_noop("\\svnInfoLongDate")?;
  def_macro_noop("\\svnInfoMaxDay")?;
  def_macro_noop("\\svnInfoMaxMonth")?;
  def_macro_noop("\\svnInfoMaxRevision")?;
  def_macro_noop("\\svnInfoMaxToday")?;
  def_macro_noop("\\svnInfoMaxYear")?;
  def_macro_noop("\\svnInfoMinRevision")?;
  def_macro_noop("\\svnInfoMonth")?;
  def_macro_noop("\\svnInfoOwner")?;
  def_macro_noop("\\svnInfoRevision")?;
  def_macro_noop("\\svnInfoTime")?;
  def_macro_noop("\\svnInfoYear")?;
  DefMacro!("\\svnKeywordempty", "\\relax");
  def_macro_noop("\\svnMaxToday")?;
  def_macro_noop("\\svnToday")?;
});
