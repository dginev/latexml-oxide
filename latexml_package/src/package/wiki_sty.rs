use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("wiki", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
