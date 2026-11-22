//! Red/green guards for batch LUAFONT: document Lua's `font`, `lang` and `vf` never crash the texlua child. texlua
//! ships LuaTeX's libraries without the engine's font memory or typesetter; the `lua_bridge.rs` prelude mirrors
//! `font` as one font, the current one (its characters answer `\iffontchar\font`), and makes every call that needs
//! the engine a plain Lua error. OXIDIZED_DESIGN_DIVERGENCES #1081. Self-skip without a host `texlua`.
use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

const LUATEX: Option<&str> = Some("[rawstyles,rawclasses,luatex]latexml.sty");

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/luatex-profile/",
      $name,
      ".tex"
    ))
  };
}

fn texlua_present() -> bool {
  std::process::Command::new("texlua")
    .arg("--version")
    .output()
    .is_ok_and(|o| o.status.success())
}

/// hologo's `\HOLOGO@IfCharExists` (hologo.sty:1298, its luatex branch `font.fonts[font.current()].characters[..]`)
/// asks the current font for U+018E: the XeTeX logo prints the reversed E as lualatex does ("A XƎTEX B"). Base:
/// `font.current()` crashed texlua (`Error:lua:interpreter`), and the fallback's mirrored E leaked
/// `\pdfextension literal` text ("iteralq -1 0 0 1 0 0 cm"). Witness TL manuals hvlogos/hvlogos-doc, bxwareki and 10
/// more (the repro lists them).
/// Repro luatex-profile/lua_font_mirror_hologo.
#[test]
fn batchluafont_hologo_asks_the_current_font_for_its_glyph() {
  if !texlua_present() {
    return;
  }
  let (log, xml) = convert_with(repro!("lua_font_mirror_hologo"), LUATEX);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let p = xml
    .split("<p>")
    .nth(1)
    .and_then(|p| p.split("</p>").next())
    .unwrap_or_default();
  assert!(
    p.starts_with("A X") && p.contains('\u{018E}') && p.ends_with("X B"),
    "{xml}"
  );
  assert!(!xml.contains("iteralq"), "{xml}");
}

/// The calls with no engine behind them answer without a crash: `font.id` names no font (-1), `font.define`,
/// `lang.new`, `vf.char` and texlua's `tex.finish` are a Lua error a pcall catches, `font.current`/`max`/`nextid`
/// agree on the one font, and babel's bidi Lua (`font.id('tenln')` at babel-bidi-basic.lua:44) loads without an
/// Error. Base: each crashing chunk was an `Error:lua:interpreter`, and past 3 restarts no chunk ran. Witness TL
/// manual lettrine/lettrine-demo-arabic (luababel.def:1484). One guard: the
/// shims share one code path (`needs_engine`). Repro luatex-profile/lua_font_lang_vf_need_the_engine.
#[test]
fn batchluafont_font_lang_vf_calls_need_the_engine() {
  if !texlua_present() {
    return;
  }
  let (log, xml) = convert_with(repro!("lua_font_lang_vf_need_the_engine"), LUATEX);
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(
    xml.contains("I[-1]\nD[false]\nL[false]\nV[false]\nF[false]\nC[1/1/2]\nAB"),
    "{xml}"
  );
}

/// An interpreter that ends mid-chunk is still an Error, which now says how it ended (`exit status: 3`; a crash
/// reads `signal: 11 (SIGSEGV)`), where the base said "unexpected end of file". Distinct from the SECLUA restart
/// guard, which counts the Errors: this pins the message.
#[test]
fn batchluafont_interpreter_end_says_how() {
  if !texlua_present() {
    return;
  }
  let tex = "\\documentclass{article}\\begin{document}A\\directlua{os.exit(3)}B\\end{document}";
  let (log, xml) = convert_with(tex, LUATEX);
  let ended: Vec<&str> = log
    .lines()
    .filter(|line| line.starts_with("Error:lua:interpreter "))
    .collect();
  assert_eq!(ended.len(), 1, "{log}");
  assert!(
    ended[0].contains("the texlua interpreter ended (exit status: 3)"),
    "{log}"
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(xml.contains("AB"), "{xml}");
}
