use crate::prelude::*;

// translator.sty — string-translation framework used by tikz, beamer,
// glossaries (231 lines plain TeX). Perl LaTeXML has no
// translator.sty.ltxml — raw-loads. Rust shim forces raw-load.
// Part of the glossaries dependency chain; raw-loads cleanly
// (0 errors when probed).

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("translator", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
