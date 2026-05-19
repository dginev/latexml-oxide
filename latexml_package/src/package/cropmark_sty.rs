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
  DefRegister!("\\croplength" => Dimension::new(0));
  DefRegister!("\\cropwidth" =>  Dimension::new(0));
  DefRegister!("\\cropsep" =>    Dimension::new(0));
  DefRegister!("\\croppadtop" => Dimension::new(0));
  DefRegister!("\\croppadbot" => Dimension::new(0));
  DefRegister!("\\croppadlr" =>  Dimension::new(0));
  def_macro_noop("\\thispagecropped")?;
  def_macro_noop("\\allpagescropped")?;
  def_macro_noop("\\nopagecropped")?;
  DefConditional!("\\ifbottomcrops", {
    true
  });
});
