//! chemmacros.sty — chemistry typesetting (expl3-based).
//!
//! Raw-loaded: chemmacros.sty is a large expl3 package (`\ch` via
//! chemformula/mhchem, `\ox`, `\state`, `\NMR`, `\iupac`, …) and the real
//! file runs cleanly on the batch-54 kernel (chemmacros manual; the earlier
//! stub's `\ch` → `\ensuremath{\mathrm{#1}}` overrode chemformula's `\ch`
//! whenever both were loaded — chemformula manual: 59 `\lx@end@inline@math`
//! + 32 malformed `ltx:text` errors from `\ch{CrO4^2-}` typeset as math).
//! Perl LaTeXML has no chemmacros binding and skips it under
//! INCLUDE_STYLES=false; the 2024 stub rationale (~1000 cascading expl3
//! errors per paper, witnesses 2407.06385, 2408.16742, 2408.16711) no longer
//! holds. Its `formula=chemformula` method needs the chemformula l3 API,
//! which the chemformula binding provides (`\chemformula_chcpd:nn` …).
//! Guard: `perfect_kernel_batch54::chemmacros_raw_load_keeps_chemformula_ch`.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("chemmacros", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
