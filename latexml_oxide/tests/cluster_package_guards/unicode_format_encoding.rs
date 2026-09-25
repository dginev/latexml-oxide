//! A Unicode-engine format (the `luatex`/`xetex` profiles) makes TU the default
//! encoding (fonttext.ltx:57-68,93): text and `\char` decode as Unicode code
//! points, through the TeX ligatures its Latin Modern fonts load with
//! (tuenc.def:60/100). The pdfTeX-model default keeps OT1. Repro
//! `tools/perfect_kernel/repros/unicode-catcodes/tu_encoding_char_and_tlig.tex`
//! (glossaries-user, latexbangla); expected output is lualatex's and
//! xelatex's, identical.
use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

const TEX: &str = include_str!(
  "../../../tools/perfect_kernel/repros/unicode-catcodes/tu_encoding_char_and_tlig.tex"
);

const UNICODE_P: &str = "<p>A{x}B <text font=\"typewriter\">C{y}D</text> \\_&lt;|”lt &lt; gt &gt; bar | quote ” dash – em — “q” <text font=\"bold\">b&lt;</text> <text font=\"sansserif\">s&lt;</text>\n[TU][TU]</p>";

fn paragraph(preload: &str) -> String {
  let (stderr, xml) = convert_with(TEX, Some(preload));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let start = xml.find("<p>").unwrap_or_else(|| panic!("no <p>: {xml}"));
  let end = xml[start..]
    .find("</p>")
    .map(|e| start + e + 4)
    .unwrap_or(xml.len());
  xml[start..end].to_string()
}

/// Under the `luatex` profile `\char`\{` is "{", a typed `<` is "<", and `"`
/// `'` `` ` `` pass through the TeX ligatures; typewriter has none.
#[test]
fn luatex_profile_decodes_text_as_tu() {
  assert_eq!(
    paragraph("[rawstyles,rawclasses,luatex]latexml.sty"),
    UNICODE_P
  );
}

/// The `xetex` profile installs the same format encoding (tuenc.def's XeTeX
/// branch, taken once `\XeTeXrevision` is defined).
#[test]
fn xetex_profile_decodes_text_as_tu() {
  assert_eq!(
    paragraph("[rawstyles,rawclasses,xetex]latexml.sty"),
    UNICODE_P
  );
}

/// Control: the pdfTeX-model format keeps OT1, as pdflatex prints it
/// (`\char`\{` in cmr is the en dash).
#[test]
fn pdftex_profile_keeps_ot1() {
  assert_eq!(
    paragraph("[rawstyles,rawclasses]latexml.sty"),
    "<p>A–x˝B <text font=\"typewriter\">C{y}D</text> “˙¡—”lt ¡ gt ¿ bar — quote ” dash – em — “q” <text font=\"bold\">b¡</text> <text font=\"sansserif\">s¡</text>\n[OT1][OT1]</p>"
  );
}
