use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: kvoptions.sty.ltxml — pure raw-load passthrough.
  InputDefinitions!("kvoptions", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
