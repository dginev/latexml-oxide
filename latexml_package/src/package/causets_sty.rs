use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: causets.sty.ltxml (#2737, eb08bd7f) — causets is a TikZ extension.
  // The binding is a pure raw-load passthrough (its upstream RequirePackage calls
  // were removed); load the host's real causets.sty, never an .ltxml.
  InputDefinitions!("causets", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
