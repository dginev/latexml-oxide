use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: upquote.sty.ltxml
  InputDefinitions!("upquote", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
