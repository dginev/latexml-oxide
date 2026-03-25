use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefRegister!("\\croplength" => Dimension::new(0));
  DefRegister!("\\cropwidth" =>  Dimension::new(0));
  DefRegister!("\\cropsep" =>    Dimension::new(0));
  DefRegister!("\\croppadtop" => Dimension::new(0));
  DefRegister!("\\croppadbot" => Dimension::new(0));
  DefRegister!("\\croppadlr" =>  Dimension::new(0));
  DefMacro!("\\thispagecropped", "");
  DefMacro!("\\allpagescropped", "");
  DefMacro!("\\nopagecropped",   "");
  DefConditional!("\\ifbottomcrops", {
    true
  });
});
