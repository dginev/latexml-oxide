use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: bbding.sty.ltxml
  // See the font map ding.fontmap.ltxml for the Unicode equivalences.
  InputDefinitions!("bbding", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
