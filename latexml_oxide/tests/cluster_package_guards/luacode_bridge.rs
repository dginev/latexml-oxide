//! The texlua bridge (`latexml_engine::lua_bridge`) + luacode.sty binding:
//! `{luacode}` bodies and `\luaexec` chunks execute in a persistent
//! external texlua, and their `tex.print` output re-enters the TeX stream.
//! Self-skips without a host `texlua` (CI trimmed-TL trap: a green run on
//! such a host does not prove the bridge ran).

use std::process::Command;

const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{luacode}\n\
    \\begin{document}\n\
    E:\\luaexec{tex.print(3+4)}.\n\
    \\begin{luacode}\n\
    local sum = 0\n\
    for i = 1, 10 do sum = sum + i end\n\
    tex.print(\"Sum: \" .. sum)\n\
    \\end{luacode}\n\
    after\n\
    \\end{document}\n";

#[test]
fn luacode_executes_via_texlua() {
  if !Command::new("texlua")
    .arg("--version")
    .output()
    .is_ok_and(|o| o.status.success())
  {
    return; // no texlua on this host
  }
  let (stderr, xml) = super::convert(TEX, false);
  assert!(
    !stderr.contains("Error:"),
    "luacode must digest cleanly:\n{stderr}"
  );
  assert!(
    xml.contains("E:7") && xml.contains("Sum: 55") && xml.contains("after"),
    "lua output and following content must both survive:\n{xml}",
  );
}
