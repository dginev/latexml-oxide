//! codehigh.sty — raw load, with the LuaTeX code parser degraded to plain
//! (unhighlighted) text.
//!
//! codehigh.sty:508 dispatches on `\legacy_if:nTF {LuaTeX}`: under the
//! `luatex` profile it takes `\__cdhh_parse_code_luatex:nN` (:575-586), whose
//! `\directlua{ParseCode(token.scan_argument(),…)}` needs the LuaTeX runtime
//! `token` library our texlua bridge cannot supply, so the Lua-defined
//! `\l__cdhh_parse_code_count_tl` never existed (CreationBoites-doc,
//! tkz-bernoulli-doc, tabularray-abnt-pt-br, functional). Pinning the
//! l3regex parser (:514) instead highlighted correctly but is O(n²) in the
//! code length (PLANS P65) — all four manuals ran past the 300 s sweep
//! timeout (sweep #34). So the LuaTeX path typesets the code as one
//! unstyled chunk (style `0`, what the normal parser emits for non-matching
//! text): the listing is kept verbatim, without colour, in bounded time. The
//! non-LuaTeX path (codehigh's own manual under pdflatex) is untouched.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("codehigh", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r"\ExplSyntaxOn
\cs_set_protected:Npn \__cdhh_parse_code_luatex:nN #1 #2 { \__cdhh_typeset_text:nN {0} #2 }
\ExplSyntaxOff"
  );
});
