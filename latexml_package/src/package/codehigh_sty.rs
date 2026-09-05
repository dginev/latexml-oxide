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
//! non-LuaTeX parser (`\__cdhh_parse_code_normal:nN`, codehigh.sty:514-570:
//! `\bool_do_until` re-running `\regex_extract_once`/`\regex_split` on the
//! whole remaining code per rule) has the same O(rules·n²) shape, so
//! `\dochighinput{<package>.sty}` of a 1000-line source (fontscale-code,
//! scaletextbullet-code, polyomino, pegmatch, liftarm, broydensolve-doc,
//! codehigh's own manual — measured 3 s load, 65 s for 60 lines, timeout at
//! 200) ran past every limit; it degrades the same way. pdflatex is fast and
//! clean; Perl raw-loads the same regex engine and times out too. Guard:
//! `perfect_kernel_batch56::codehigh_dochighinput_is_bounded`.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("codehigh", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r"\ExplSyntaxOn
\cs_set_protected:Npn \__cdhh_parse_code_luatex:nN #1 #2 { \__cdhh_typeset_text:nN {0} #2 }
\cs_set_protected:Npn \__cdhh_parse_code_normal:nN #1 #2 { \__cdhh_typeset_text:nN {0} #2 }
\ExplSyntaxOff"
  );
});
