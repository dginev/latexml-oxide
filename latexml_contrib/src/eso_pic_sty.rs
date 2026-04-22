use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("xcolor");
  RequirePackage!("keyval");
  // Perl-parity stubs (matches ar5iv-bindings/eso-pic.sty.ltxml L21-38
  // exactly): all 17 shipout/grid CSes are `Tokens()` no-ops in Perl
  // too. \LenToUnit is the one identity passthrough.
  DefMacro!("\\AddToShipoutPicture OptionalMatch:* {}", "");
  DefMacro!("\\AddToShipoutPictureBG OptionalMatch:* {}", "");
  DefMacro!("\\AddToShipoutPictureFG OptionalMatch:* {}", "");
  DefMacro!("\\AtPageCenter OptionalMatch:* {}", "");
  DefMacro!("\\AtPageLowerLeft OptionalMatch:* {}", "");
  DefMacro!("\\AtPageUpperLeft OptionalMatch:* {}", "");
  DefMacro!("\\AtStockCenter OptionalMatch:* {}", "");
  DefMacro!("\\AtStockLowerLeft OptionalMatch:* {}", "");
  DefMacro!("\\AtStockUpperLeft OptionalMatch:* {}", "");
  DefMacro!("\\AtTextCenter OptionalMatch:* {}", "");
  DefMacro!("\\AtTextLowerLeft OptionalMatch:* {}", "");
  DefMacro!("\\AtTextUpperLeft OptionalMatch:* {}", "");
  DefMacro!("\\ClearShipoutPicture", "");
  DefMacro!("\\ClearShipoutPictureBG", "");
  DefMacro!("\\ClearShipoutPictureFG", "");
  DefMacro!("\\LenToUnit{}", "#1");
  DefMacro!("\\ProcessOptionsWithKV{}", "");
  DefMacro!("\\gridSetup[]{}{}{}{}{}", "");
});
