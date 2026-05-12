use crate::prelude::*;

// datatool-base.sty — shared kernel of the datatool family. Driver
// for `datatool`, used transitively by `glossaries` (via the long /
// super / list / tree glossary styles). 3429 lines, expl3-light
// (mostly \def/\edef + xkeyval).
//
// Deps: etoolbox, amsmath, xkeyval, xfor, ifthen, substr,
// datatool-<mathprocessor>.
//
// Third step of the SYNC_STATUS "raw-load enablement" plan
// (after xfor + mfirstuc): forward to the TL `.sty` so we can
// profile gaps surfacing from substr.sty + datatool-fp/pgfmath.

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("datatool-base", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
