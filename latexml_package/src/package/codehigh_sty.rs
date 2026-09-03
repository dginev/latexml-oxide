//! codehigh.sty — raw load, with the code parser pinned to codehigh's own
//! l3regex path.
//!
//! codehigh.sty:508 dispatches on `\legacy_if:nTF {LuaTeX}`: under the
//! `luatex` profile it takes `\__cdhh_parse_code_luatex:nN` (:575-586), whose
//! `\directlua{ParseCode(token.scan_argument(),…)}` needs the LuaTeX runtime
//! `token` library our texlua bridge cannot supply, so the Lua-defined
//! `\l__cdhh_parse_code_count_tl` never exists (CreationBoites-doc,
//! tkz-bernoulli-doc, tabularray-abnt-pt-br, functional). The normal path
//! (:514) produces the same highlighting on every non-LuaTeX engine.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("codehigh", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r"\ExplSyntaxOn
\cs_set_protected:Npn \__cdhh_parse_code:nN #1 #2 { \__cdhh_parse_code_normal:nN {#1} #2 }
\ExplSyntaxOff"
  );
});
