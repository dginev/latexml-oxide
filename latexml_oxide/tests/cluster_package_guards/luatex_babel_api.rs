//! Under the `luatex` profile, babel's Lua API layer (luababel.def L196+,
//! creating `Babel.locale_props`, `Babel.lua_error`, …) must actually run.
//! In a real lualatex job `\bbl@luapatterns` lives in the FORMAT, so
//! babel.def L1135 skips the patterns-only first `\input luababel.def`
//! and the single in-document load (babel.def L2285) takes the API
//! branch. Without that format fact, the patterns-only load ran first,
//! `\endinput`ed at luababel.def L195, and the loaded-flag suppressed the
//! second `\input` — so every later `Babel.locale_props[...]` chunk died.
//! Systemic witnesses: every profiled clean-lualatex corpus doc logged
//! `attempt to index a nil value (field 'locale_props')` (abntexto,
//! abntexto-uece, derivative, newpax). Self-skips without texlua.

use std::process::Command;

const TEX: &str = "\\documentclass{article}\n\
    \\usepackage[english]{babel}\n\
    \\makeatletter\n\
    \\begin{document}\n\
    \\lx@directlua{tex.sprint(Babel and Babel.locale_props and 'BOK' or 'BNO')}\n\
    \\end{document}\n";

#[test]
fn babel_lua_api_layer_initializes() {
  if !Command::new("texlua")
    .arg("--version")
    .output()
    .is_ok_and(|o| o.status.success())
  {
    return; // no texlua on this host
  }
  let (stderr, xml) = super::convert_with(TEX, Some("[luatex]latexml.sty"));
  assert!(
    xml.contains("BOK"),
    "Babel.locale_props must exist after babel loads under the luatex profile:\n{xml}\n{stderr}",
  );
}
