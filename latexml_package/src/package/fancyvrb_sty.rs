use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: fancyvrb.sty.ltxml
  InputDefinitions!("fancyvrb", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
