use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ed.sty.ltxml
  // this just works and produces reasonable output
  InputDefinitions!("ed", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
