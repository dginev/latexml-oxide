use crate::prelude::*;

// tracklang.sty — language tracking and identification (215 lines).
// Perl LaTeXML has no tracklang.sty.ltxml — raw-loads. Rust shim
// forces raw-load. Part of the glossaries dependency chain;
// raw-loads cleanly (0 errors when probed).

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("tracklang", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
