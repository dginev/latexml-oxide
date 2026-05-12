use crate::prelude::*;

// mfirstuc.sty — make-first-uppercase utility shipped with glossaries.
// 664 lines, expl3-heavy (\tl_if_empty:nF, \cs_new:Npn, \l__mfirstuc_*_tl).
// Perl LaTeXML has no `mfirstuc.sty.ltxml`; glossaries.sty.ltxml's
// `RequirePackage('xfor')` plus auto-dep scan causes Perl to raw-load
// mfirstuc.sty directly.
//
// Second step of the SYNC_STATUS "raw-load enablement" plan
// (after xfor): forward to the TL `.sty` so we can profile which
// expl3 primitives are still missing on the Rust side.

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("mfirstuc", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
