//! OXIDIZED_DESIGN #168: the opt-in `luatex` latexml.sty option flips the
//! document to LuaTeX identity — iftex probes consult LUATEX_PROFILE state
//! (immune to load order), and `\directlua` exists under its REAL name for
//! that conversion only. Without the option the pdfTeX-model identity is
//! untouched (defining \directlua by default flipped 26 tests onto luatex
//! paths — the regression this guard pins).

const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{iftex}\n\
    \\begin{document}\n\
    engine:\\iftutex LUA\\else PDF\\fi. \
    dl:\\ifdefined\\directlua DEF\\else UNDEF\\fi.\n\
    \\end{document}\n";

fn convert(preload: &str) -> String { super::convert_with(TEX, Some(preload)).1 }

#[test]
fn profile_flips_identity_only_when_opted_in() {
  let on = convert("[rawstyles,rawclasses,luatex]latexml.sty");
  assert!(
    on.contains("engine:LUA") && on.contains("dl:DEF"),
    "[luatex] must flip iftex probes and expose \\directlua:\n{on}",
  );
  let off = convert("[rawstyles,rawclasses]latexml.sty");
  assert!(
    off.contains("engine:PDF") && off.contains("dl:UNDEF"),
    "without [luatex] the pdfTeX identity must be untouched:\n{off}",
  );
}
