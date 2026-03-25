use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: algpascal.sty.ltxml — loads hacked algorithmicx
  InputDefinitions!("algpascal", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
