//! pgfutil-common.tex — PGF utility macros
//! Perl: pgfutil-common.tex.ltxml (38 lines)
//!
//! Loads the raw TeX code for pgf utilities.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L22: Load pgf's TeX code for util-common first
  InputDefinitions!("pgfutil-common", extension => Some(Cow::Borrowed("tex")), noltxml => true);
  // pgfutil-common.tex:867-882 keys its engine branch on `\directlua` being
  // defined and, on LuaTeX, lets Lua create `\pgfutil@luaescapestring`
  // (`tex.enableprimitives`); under the `[luatex]` profile `\directlua` is
  // the engine identity (never redefined) but evaluates no Lua, so the name
  // stayed undefined for pgf's luamath/graphdrawing libraries
  // (pgflibrarygraphdrawing.code.tex:146, pgflibraryluamath.code.tex:126;
  // neoschool ×2, beamerthemeCelestia ×2). pgf's own non-LuaTeX fallback
  // (:882, luamath :68) is the TeX no-op; the pdflatex profile already
  // defines the same. Guard: `perfect_kernel_batch54::pgf_lua_entry_points_have_their_tex_fallback`.
  RawTeX!(r"\def\pgfutil@luaescapestring#1{}");
});
