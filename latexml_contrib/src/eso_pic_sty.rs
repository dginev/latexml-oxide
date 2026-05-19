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
  RequirePackage!("xcolor");
  RequirePackage!("keyval");
  // Perl-parity stubs (matches ar5iv-bindings/eso-pic.sty.ltxml L21-38
  // exactly): all 17 shipout/grid CSes are `Tokens()` no-ops in Perl
  // too. \LenToUnit is the one identity passthrough.
  def_macro_noop("\\AddToShipoutPicture OptionalMatch:* {}")?;
  def_macro_noop("\\AddToShipoutPictureBG OptionalMatch:* {}")?;
  def_macro_noop("\\AddToShipoutPictureFG OptionalMatch:* {}")?;
  def_macro_noop("\\AtPageCenter OptionalMatch:* {}")?;
  def_macro_noop("\\AtPageLowerLeft OptionalMatch:* {}")?;
  def_macro_noop("\\AtPageUpperLeft OptionalMatch:* {}")?;
  def_macro_noop("\\AtStockCenter OptionalMatch:* {}")?;
  def_macro_noop("\\AtStockLowerLeft OptionalMatch:* {}")?;
  def_macro_noop("\\AtStockUpperLeft OptionalMatch:* {}")?;
  def_macro_noop("\\AtTextCenter OptionalMatch:* {}")?;
  def_macro_noop("\\AtTextLowerLeft OptionalMatch:* {}")?;
  def_macro_noop("\\AtTextUpperLeft OptionalMatch:* {}")?;
  def_macro_noop("\\ClearShipoutPicture")?;
  def_macro_noop("\\ClearShipoutPictureBG")?;
  def_macro_noop("\\ClearShipoutPictureFG")?;
  DefMacro!("\\LenToUnit{}", "#1");
  def_macro_noop("\\ProcessOptionsWithKV{}")?;
  def_macro_noop("\\gridSetup[]{}{}{}{}{}")?;
});
