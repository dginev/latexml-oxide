use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: cmap.sty.ltxml
  // The cmap package makes "search" and "copy-and-paste" functions work properly
  // in pdfs. There's nothing we need to do.
  DeclareOption!("resetfonts", None);
  ProcessOptions!();
});
