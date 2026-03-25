use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("xkeyval", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  InputDefinitions!("xkvview", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
