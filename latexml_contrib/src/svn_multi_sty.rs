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
  Warn!(
    "missing_file",
    "svn-multi.sty",
    "svn-multi.sty is not implemented and will not be interpreted raw."
  );
  RequirePackage!("graphics");
  DefConditional!("\\ifsvnfilemodified");
  DefConditional!("\\ifsvnmodified");
  DefConditional!("\\ifsvnsubgroups");
  def_macro_noop("\\svnFullAuthor OptionalMatch:* {}")?;
  def_macro_noop("\\svnRegisterAuthor{}{}")?;
  def_macro_noop("\\svnRegisterRevision OptionalMatch:* {}{}")?;
  def_macro_noop("\\svncgtime")?;
  def_macro_noop("\\svncgtimezone")?;
  def_macro_noop("\\svncgtoday")?;
  def_macro_noop("\\svndate")?;
  def_macro_noop("\\svnday")?;
  def_macro_noop("\\svnfileauthor")?;
  def_macro_noop("\\svnfiledate")?;
  def_macro_noop("\\svnfileday")?;
  def_macro_noop("\\svnfiledir")?;
  def_macro_noop("\\svnfilefname")?;
  def_macro_noop("\\svnfilehour")?;
  def_macro_noop("\\svnfileminute")?;
  def_macro_noop("\\svnfilemonth")?;
  def_macro_noop("\\svnfilerev")?;
  def_macro_noop("\\svnfilesecond")?;
  def_macro_noop("\\svnfiletime")?;
  def_macro_noop("\\svnfiletimezone")?;
  def_macro_noop("\\svnfiletimezonehour")?;
  def_macro_noop("\\svnfiletimezoneminute")?;
  def_macro_noop("\\svnfiletoday")?;
  def_macro_noop("\\svnfileyear")?;
  def_macro_noop("\\svnhour")?;
  def_macro_noop("\\svnid{}")?;
  def_macro_noop("\\svnidlong")?;
  def_macro_noop("\\svnminute")?;
  DefMacro!("\\svnnolinkurl", "#1");
  def_macro_noop("\\svnsecond")?;
  def_macro_noop("\\svntime")?;
  def_macro_noop("\\svntimezone")?;
  def_macro_noop("\\svntimezonehour")?;
  def_macro_noop("\\svntimezoneminute")?;
  def_macro_noop("\\svntoday")?;
  def_macro_noop("\\svnurl{}")?;
  def_macro_noop("\\svnyear")?;
  def_macro_noop("\\tableofrevisions")?;
  DefEnvironment!("{svnfilerow}", "");
  DefEnvironment!("{svnglobalrow}", "");
  DefEnvironment!("{svngrouprow}", "");
  DefEnvironment!("{svnsubgrouprow}", "");
  DefEnvironment!("{svntable}", "");
});
