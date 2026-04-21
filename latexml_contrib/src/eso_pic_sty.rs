use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("xcolor");
  RequirePackage!("keyval");
  // INCOMPLETE IMPLEMENTATION — just a stub ignoring the functionality for now
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
