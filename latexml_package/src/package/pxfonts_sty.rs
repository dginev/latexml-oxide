use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pxfonts.sty.ltxml
  // As far as LaTeXML is concerned, these are identical — but pxfonts.sty has no `\varmathbb`
  // (txfonts.sty:920), so a txfonts loaded only for pxfonts leaves it undefined.
  let txfonts_before = lookup_bool("txfonts.sty_loaded");
  RequirePackage!("txfonts");
  if !txfonts_before {
    assign_meaning(&T_CS!("\\varmathbb"), Stored::None, None);
  }
});
