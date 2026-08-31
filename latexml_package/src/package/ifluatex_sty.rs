//! ifluatex.sty — LuaTeX detection. Mirrors iftex_sty.rs: TRUE under the
//! opt-in `luatex` latexml.sty profile, false otherwise. A hardcoded false
//! here silently RE-clobbered the `\let\ifluatex\iftrue` the profile
//! installed, because these legacy shims are `\RequirePackage`'d by the very
//! font packages that branch on them (raleway.sty:36, sourcecodepro.sty:34,
//! AlegreyaSans.sty:9 — their `\RequirePackage{fontspec}` at :71/:68/:164
//! never fired; 68 of the 300 luatex-profile corpus docs load this shim).
//! Witnesses: parnotes, raleway, sourcecodepro, sourcesanspro (all oracle
//! lualatex-clean). Perl: ifluatex.sty.ltxml (always false — Perl has no
//! LuaTeX identity at all).
use crate::prelude::*;

LoadDefinitions!({
  DefConditional!("\\ifluatex", { lookup_bool("LUATEX_PROFILE") });
});
