use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: algc.sty.ltxml — loads hacked algorithmicx
  InputDefinitions!("algc", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
