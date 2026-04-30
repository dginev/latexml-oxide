use crate::prelude::*;
LoadDefinitions!({
  // Perl: expl3.ltx.ltxml — used when processing latex.ltx (for
  // dumping). Loads raw expl3.ltx then marks both the .sty and .lua
  // bindings "loaded" so subsequent \usepackage{expl3} etc. are no-ops.
  InputDefinitions!("expl3", extension => Some("ltx".into()), noltxml => true);
  AssignValue!("expl3.sty_loaded" => 1, Some(Scope::Global));
  AssignValue!("expl3.lua_loaded" => 1, Some(Scope::Global));
});
