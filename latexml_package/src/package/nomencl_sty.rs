use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: nomencl.sty.ltxml
  InputDefinitions!("nomencl", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
