use crate::prelude::*;

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
