//! `tex.count`/`tex.dimen` reads AND writes inside `\directlua` chunks are
//! LIVE against engine State, via the bridge's query protocol. This is the
//! "rebind-as-we-emulate" seam (docs/perfect_kernel/LUA_REBINDING.md):
//! texlua has no engine, so any tex-state access a chunk makes must
//! round-trip to OUR State — the previous stub returned zeros, which made
//! every register-branching Lua chunk take the wrong path silently.
//! Systemic witness class: babel's luababel.def chunks under the `luatex`
//! profile (every profiled doc logged `attempt to index a nil value
//! (field 'locale_props')` — chunks die mid-sequence, later chunks see
//! missing state). Self-skips without a host texlua.

use std::process::Command;

const TEX: &str = "\\documentclass{article}\n\
    \\makeatletter\n\
    \\begin{document}\n\
    \\count255=7 \\dimen0=2pt\n\
    \\lx@directlua{tex.count[100] = tex.getcount(255) + 35\n\
      tex.sprint('C' .. tex.count[255] .. 'D' .. tex.dimen[0])}\n\
    E\\the\\count100.\n\
    \\end{document}\n";

#[test]
fn directlua_reads_and_writes_live_registers() {
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
    "directlua must digest cleanly:\n{stderr}"
  );
  // \count255=7 read back; \dimen0=2pt as 131072 sp (LuaTeX convention:
  // tex.dimen reads in scaled points); the Lua-side write of count 100
  // visible to the following \the.
  assert!(
    xml.contains("C7D131072") && xml.contains("E42."),
    "live register mirror must round-trip both directions:\n{xml}",
  );
}
