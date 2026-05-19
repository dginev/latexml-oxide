use crate::prelude::*;

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
