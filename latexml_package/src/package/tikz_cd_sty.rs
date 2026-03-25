use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: tikz-cd.sty.ltxml
  InputDefinitions!("tikz-cd", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
