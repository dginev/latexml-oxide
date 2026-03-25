use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lipsum.sty.ltxml
  RequirePackage!("xparse");
  InputDefinitions!("lipsum", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
