use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: fixme.sty.ltxml
  InputDefinitions!("fixme", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
