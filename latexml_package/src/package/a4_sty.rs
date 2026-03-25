use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: a4.sty.ltxml
  DefMacro!("\\WideMargins", "");
  DefRegister!("\\ExtraWidth" => Dimension::new(0));
});
