use latexml_package::prelude::*;

LoadDefinitions!({
  // ltluatex.tex is the standalone copy of the lualatex FORMAT surface
  // (latex.ltx:896-1058), which the `luatex` profile provides
  // (latexml_sty/mod.rs). Packages reach it by a plain `\input ltluatex`
  // (luaotfload.sty:38) that bypasses this registry and runs the raw file —
  // idempotent once the primitives exist. Nothing to do here.
});
