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
  DefRegister!("\\PaperWidth" =>  Dimension::new(0));
  DefRegister!("\\PaperHeight" => Dimension::new(0));
  DefConditional!("\\ifLandscape");
  def_macro_noop("\\setpapersize[]{}")?;
  def_macro_noop("\\setmargins{}{}{}{}{}{}{}{}")?;
  def_macro_noop("\\setmarginsrb{}{}{}{}{}{}{}{}")?;
  def_macro_noop("\\setmargnohf{}{}{}{}")?;
  def_macro_noop("\\setmargnohfrb{}{}{}{}")?;
  def_macro_noop("\\setmarg{}{}{}{}")?;
  def_macro_noop("\\setmargrb{}{}{}{}")?;
  def_macro_noop("\\margin@offset")?;
  def_macro_noop("\\shiftmargins")?;
});
