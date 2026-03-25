use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("ltxcmds", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
