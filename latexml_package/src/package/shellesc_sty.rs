use crate::prelude::*;

// shellesc.sty — shell-escape detection helpers (132 lines plain TeX).
// Perl LaTeXML has no shellesc.sty.ltxml — relies on raw-load. Rust
// shim forces raw-load via `noltxml=>true`. Part of the glossaries
// dependency chain; raw-loads cleanly (0 errors when probed).

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("shellesc", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
