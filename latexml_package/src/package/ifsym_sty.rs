use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ifsym.sty.ltxml
  // See the font maps (ifblk, ifclk, ifgeo, ifsym, ifwea).fontmap.ltxml for the Unicode equivalences.
  InputDefinitions!("ifsym", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
