use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: algcompatible.sty.ltxml — loads hacked algorithmicx
  InputDefinitions!("algcompatible", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
