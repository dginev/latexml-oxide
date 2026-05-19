use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: a4.sty.ltxml
  def_macro_noop("\\WideMargins")?;
  DefRegister!("\\ExtraWidth" => Dimension::new(0));
});
