use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: algmatlab.sty.ltxml — loads hacked algorithmicx
  InputDefinitions!("algmatlab", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
