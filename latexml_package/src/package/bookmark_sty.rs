use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: bookmark.sty.ltxml
  InputDefinitions!("bookmark", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
