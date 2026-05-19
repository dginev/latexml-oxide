use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  Warn!(
    "missing_file",
    "datetime2.sty",
    "datetime2.sty is only minimally stubbed and will not be interpreted raw."
  );
  // datetime2 L518: \DTMcurrenttime / \DTMnow / \DTMcurrentdate.
  // We don't render time placeholders; gobble.
  def_macro_noop("\\DTMcurrenttime")?;
  def_macro_noop("\\DTMnow")?;
  def_macro_noop("\\DTMcurrentdate")?;
  DefMacro!("\\DTMtoday", "\\today");
  def_macro_noop("\\DTMusemodule{}{}")?;
  def_macro_noop("\\DTMsetdatestyle{}")?;
  def_macro_noop("\\DTMsetstyle{}")?;
  def_macro_noop("\\DTMlangsetup[]{}")?;
  def_macro_noop("\\DTMnewstyle{}{}{}{}")?;
});
