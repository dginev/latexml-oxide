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
  assert!(
    xml.contains(
      "<p>A X<text yoffset=\"-2.2pt\">\u{018E}</text>T<text yoffset=\"-2.2pt\">E</text>X B</p>"
    ),
    "{xml}"
  );
}

/// The calls with no engine behind them answer without a crash: `font.id` answers -1, `font.define`, `font.setfont`,
/// `lang.new`, `vf.char` and texlua's `tex.finish` are a Lua error a pcall catches, `font.getparameters` and
/// `font.frozen` answer nil as in texlua, `font.current`/`max`/`nextid` agree on the one font, and babel's bidi Lua (`font.id('tenln')` at babel-bidi-basic.lua:44) loads without an Error. Each
/// answer that is not lualatex's is a Warning: the four calls, a font field the mirror does not hold (`resources`,
/// fontawesome5's and texnegar's read), and `font.id` of a font latex.ltx defines (`\tenln`,
/// `\tenlnw`, `\tencirc`, `\tencircw`), whose id lualatex would give. Base: each crashing chunk was an `Error:lua:interpreter`, and past 3 restarts no chunk ran. Witness TL
/// manual lettrine/lettrine-demo-arabic (luababel.def:1484). One guard: the
/// shims share one code path (`needs_engine`). Repro luatex-profile/lua_font_lang_vf_need_the_engine.
#[test]
fn batchluafont_font_lang_vf_calls_need_the_engine() {
  if !texlua_present() {
    return;
  }
  let (log, xml) = convert_with(repro!("lua_font_lang_vf_need_the_engine"), LUATEX);
  let id =
    |name: &str| format!("font.id(\"{name}\") names a font whose id is not known; it answers -1");
  let expected: Vec<String> = [
    id("tenln"),
    "font.define was not run".into(),
    "lang.new was not run".into(),
    "vf.char was not run".into(),
    "tex.finish was not run".into(),
    "the current font has no field resources here".into(),
    "font.getparameters answers nil".into(),
    "font.frozen answers nil".into(),
    "font.setfont was not run".into(),
    id("tenln"),
    id("tenlnw"),
    id("tencirc"),
    id("tencircw"),
  ]
  .iter()
  .map(|what| format!("Warning:lua:engine {what} (texlua has no TeX engine fonts or typesetter)"))
  .collect();
  let warned: Vec<&str> = log
    .lines()
    .filter(|line| line.starts_with("Warning:"))
    .collect();
  assert_eq!(warned, expected, "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 13), "{log}");
  assert!(
    xml.contains(
      "<p>I[-1]\nD[false]\nL[false]\nV[false]\nF[false]\nC[1/1/2]\nR[nil]\nP[nil]\nZ[nil]\nS[false]\nAB</p>"
    ),
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

/// `\pdfextension literal` reads `\pdfliteral`'s operands with LuaTeX's modes (`[shipout]
/// [direct|page|text|raw|origin] <general text>`): the literals leave no text, as under lualatex. A keyword LuaTeX does
/// not know is a Warning and stays text, as lualatex's "unexpected use of \pdfextension" leaves it. Base: one token
/// was read, leaking "iteralq -1 0 0 1 0 0 cm", accsupp's "iteralpage/Span«/ActualText(UTF-8)»BDC", and `text{0 g}`
/// was `Error:expected:{`. Witness TL manual mfirstuc/mfirstuc-manual (accsupp spans), hologo's mirrored-E fallback
/// (bxwareki). Distinct from the keyword guard: one keyword's modes, and the unknown keyword.
/// Repro luatex-profile/pdfextension_literal_operands.
#[test]
fn batchluafont_pdfextension_literal_reads_its_operands() {
  let (log, xml) = convert_with(repro!("pdfextension_literal_operands"), LUATEX);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Warning:unexpected:\\pdfextension unexpected use of \\pdfextension"),
    "{log}"
  );
  assert!(xml.contains("<p>ABC\nD EF</p>"), "{xml}");
  assert!(xml.contains("<p>Gbogus HI</p>"), "{xml}");
}

/// Every `\pdfextension <keyword>` LuaTeX knows reads `\pdf<keyword>`'s operands (luatex85.sty:69-92 defines the
/// `\pdf…` names as it; LuaTeX's additions `lateliteral`, `linkstate`, `obj uncompressed`, `outline useobjnum`), never
/// through luatex85's own macros: the letters stay and no operand is text, as under lualatex. Base: only `literal`
/// was known, and "olorstack0 push0 g", "etmatrix1 0 0 1", "nfo /Title (x)", "apline +cmr10", … leaked. Witness TL
/// manual xfakebold/xfakebold-doc (pdfrender's `\pdfcolorstack`, 59 leaks; its `\pdfcolorstackinit page direct{…}`,
/// luatex85's `\pdffeedback colorstackinit`, leaked "page direct1 w"). Distinct from the literal guard: the keyword
/// table and each keyword's reader. Repro luatex-profile/pdfextension_keywords_luatex85.
#[test]
fn batchluafont_pdfextension_keywords_read_their_operands() {
  let (log, xml) = convert_with(repro!("pdfextension_keywords_luatex85"), LUATEX);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<p>AB CDEF\nGHIJK\nLMN\nOP Q\nRSU\nVWXY\nZ abc\nde\nfg\nhij\nkl\nmnopq</p>"),
    "{xml}"
  );
}

/// `font.read_tfm` and `font.read_vf` read a TFM or VF through kpathsea, as lualatex does (the prelude sets the
/// program name, without which texlua aborts; no MKTEX* program runs). Base: both were wrapped as needing the engine
/// (`T[]`, a Warning). Witness: the pk/LUAFONT review's probe. Distinct from the mirror guards: the calls that pass
/// through to texlua. Repro luatex-profile/lua_font_read_tfm_vf.
#[test]
fn batchluafont_font_read_tfm_and_read_vf_pass_through() {
  if !texlua_present() {
    return;
  }
  let (log, xml) = convert_with(repro!("lua_font_read_tfm_vf"), LUATEX);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<p>T[cmr10]\nV[ptmr8r]</p>"), "{xml}");
}
