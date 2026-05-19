use latexml_package::prelude::*;


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
