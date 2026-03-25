use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: expl3.sty.ltxml
  LoadPool!("LaTeX");
  InputDefinitions!("expl3", extension => Some(Cow::Borrowed("lua")));
  InputDefinitions!("expl3", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
