use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: doi.sty.ltxml
  InputDefinitions!("doi", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
