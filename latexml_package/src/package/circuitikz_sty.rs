use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("circuitikz", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
