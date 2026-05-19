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
    "scrbook.cls",
    "scrbook.cls is only minimally stubbed and will not be interpreted raw."
  );
  LoadClass!("OmniBus");
  def_macro_noop("\\setkomafont{}{}")?;
  def_macro_noop("\\setcapindent{}")?;
  def_macro_noop("\\deffootnote[]{}{}{}")?;
  def_macro_noop("\\deffootnotemark{}")?;
});
