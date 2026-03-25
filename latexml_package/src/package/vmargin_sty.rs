use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefRegister!("\\PaperWidth" =>  Dimension::new(0));
  DefRegister!("\\PaperHeight" => Dimension::new(0));
  DefConditional!("\\ifLandscape");
  DefMacro!("\\setpapersize[]{}", "");
  DefMacro!("\\setmargins{}{}{}{}{}{}{}{}",   "");
  DefMacro!("\\setmarginsrb{}{}{}{}{}{}{}{}", "");
  DefMacro!("\\setmargnohf{}{}{}{}",          "");
  DefMacro!("\\setmargnohfrb{}{}{}{}",        "");
  DefMacro!("\\setmarg{}{}{}{}",              "");
  DefMacro!("\\setmargrb{}{}{}{}",            "");
  DefMacro!("\\margin@offset",                "");
  DefMacro!("\\shiftmargins",                 "");
});
