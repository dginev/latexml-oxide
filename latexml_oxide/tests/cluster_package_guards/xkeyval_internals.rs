//! Packages built on xkeyval clone `\setkeys`' front-end out of xkeyval.tex
//! and drive our key machinery through the raw internals: chessboard.sty
//! L98-107 (`\XKV@testopta{\XKV@testoptc\board@XKVsetsinglekeys}` …
//! `\XKV@s@tkeys`), xkeymask.sty (`\XKV@setkeys`, `\XKV@tempc`), xskak.
//! Sweep-12 cluster: chessboard ×3, xskak ×2, xkeymask — hundreds of
//! cascaded `\csname` errors each. The binding carries the scaffolding
//! verbatim from xkeyval.tex/xkvutils.tex, with `\XKV@s@tkeys` as a thin
//! shim onto the Rust `\setkeys` path.

const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{xkeyval}\n\
    \\makeatletter\n\
    \\define@key[UF]{fam}{mykey}{\\def\\myresult{got:#1}}\n\
    \\def\\my@do{\\XKV@testopta{\\XKV@testoptc\\my@XKVsetkeys}}\n\
    \\def\\my@XKVsetkeys[#1]#2{%\n\
      \\XKV@checksanitizea{#2}\\XKV@resb\n\
      \\let\\XKV@naa\\@empty\n\
      \\XKV@for@o\\XKV@resb\\XKV@tempa{%\n\
        \\expandafter\\XKV@g@tkeyname\\XKV@tempa=\\@nil\\XKV@tempa\n\
        \\XKV@addtolist@x\\XKV@naa\\XKV@tempa}%\n\
      \\expandafter\\XKV@s@tkeys\\expandafter{\\XKV@resb}{#1}}\n\
    \\begin{document}\n\
    \\my@do*[UF]{ fam }{ mykey = hello }\\myresult\n\
    \\setkeys[UF]{fam}{mykey=direct}\\myresult\n\
    \\end{document}\n";

/// The chessboard-shape front-end clone must parse `*`/`[prefix]{fams}`
/// through the raw `\XKV@testopta/c` scaffolding and land in the Rust
/// `\setkeys` via the `\XKV@s@tkeys` shim, with keyval.tex's
/// `\KV@@sp@def` space-trimming intact.
#[test]
fn setkeys_frontend_clone_reaches_rust_path() {
  let (stderr, xml) = super::convert(TEX, false);
  assert!(
    !stderr.contains("Error:"),
    "XKV internals must digest cleanly:\n{stderr}"
  );
  assert!(
    xml.contains("got:hello"),
    "front-end clone did not reach the key code (spaces must be trimmed):\n{xml}"
  );
  assert!(
    xml.contains("got:direct"),
    "plain \\setkeys regression:\n{xml}"
  );
}

const PTR_TEX: &str = "\\documentclass{article}\n\
    \\usepackage{xkeyval}\n\
    \\makeatletter\n\
    \\define@key[UF]{fam}{ka}{\\def\\resa{A:#1}}\n\
    \\define@key[UF]{fam}{kb}{\\def\\resb{B:#1}}\n\
    \\define@cmdkey[UF]{fam}{kc}{\\def\\resc{C:#1}}\n\
    \\def\\xkv@tokdefault{tokdef}\n\
    \\define@key[UF]{fam}{kd}[\\xkv@tokdefault]{\\def\\resd{D:#1}}\n\
    \\savekeys[UF]{fam}{\\global{ka}}\n\
    \\begin{document}\n\
    \\setkeys[UF]{fam}{ka=hello}\n\
    \\setkeys[UF]{fam}{kb=\\usevalue{ka}}\n\
    \\setkeys[UF]{fam}{kc=cval}\n\
    \\setkeys[UF]{fam}{kd}\n\
    \\makeatletter\n\
    [\\resa][\\resb][\\resc][cmd:\\cmdUF@fam@kc]\n\
    [direct:\\csname XKV@UF@fam@ka@value\\endcsname][\\resd]\n\
    \\makeatother\n\
    \\end{document}\n";

/// Three engine contracts on one doc: (1) the pointer system —
/// `\savekeys` + `\usevalue` + raw `\XKV@<header><key>@value` readback
/// (chessboard.sty L1059/L1221); (2) `\define@cmdkey` code receives the
/// BARE value as `#1` and defines `\cmd<header>` (KNOWN_PERL_ERRORS #80 —
/// Perl passes `#<value>`, leaking a PARAM token to the stomach); (3) a
/// key DEFAULT holding an internal `\xxx@yyy` name survives as one TOKEN
/// (xskak-keys.sty L25 `[\xskak@val@defaultid]` — the string round-trip
/// re-tokenized it under the standard cattable, splitting the CS and
/// cascading ~1000 csname errors per xskak/chessboard manual).
#[test]
fn pointer_system_cmdkey_and_token_defaults() {
  let (stderr, xml) = super::convert(PTR_TEX, false);
  assert!(
    !stderr.contains("Error:"),
    "pointer/cmdkey/default paths must digest cleanly:\n{stderr}"
  );
  for needle in [
    "[A:hello]",
    "[B:hello]",
    "[C:cval]",
    "[cmd:cval]",
    "[direct:hello]",
    "[D:tokdef]",
  ] {
    assert!(xml.contains(needle), "missing {needle} in:\n{xml}");
  }
}
