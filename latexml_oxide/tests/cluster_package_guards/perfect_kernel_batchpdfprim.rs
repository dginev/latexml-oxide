//! Red/green guards for batch PDFPRIM: pdfTeX's operand readers as pdftex.web reads them — one optional space after
//! `reserveobjnum`, a dest type and `newwindow` (`scan_action`'s order, `struct`), `\pdffontattr`, the font queries'
//! number and size, and the Errors TeX gives for a missing colour stack action and a missing font identifier. Each row
//! is a pdflatex/lualatex-checked repro; every Error row has a 0-error twin.
use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

const LUATEX: Option<&str> = Some("[rawstyles,rawclasses,luatex]latexml.sty");
const REPROS: &str = concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/../tools/perfect_kernel/repros/"
);

/// (repro, preload, errors, paragraph text, Error message). Distinct code paths: the pdfTeX operand readers (twin of
/// the colour stack Error), the colour stack Error, the pdfTeX font queries, `read_font_ident`'s Error (`\fontdimen`,
/// `\fontname`), and LuaTeX's `\pdfextension`/`\pdffeedback` path through the same readers.
const ROWS: &[(&str, Option<&str>, usize, &str, &str)] = &[
  (
    "backend-persona/pdftex_operands_optional_space",
    None,
    0,
    "<p>ABCDEFGHIJ KLMNOPQ R STUV</p>",
    "",
  ),
  (
    "backend-persona/pdfcolorstack_action_missing",
    None,
    1,
    "<p>AxBC D</p>",
    "Error:expected:\\pdfcolorstack Color stack action is missing",
  ),
  (
    "backend-persona/pdffont_feedback_numbers",
    None,
    0,
    "<p>AB12.0ptC10.0ptDEFXHIJ12.0ptK</p>",
    "",
  ),
  (
    "fonts-nfss/missing_font_identifier",
    None,
    2,
    "<p>A0.0ptxBnullfont3C</p>",
    "Error:expected:font identifier Missing font identifier",
  ),
  (
    "luatex-profile/pdfextension_optional_space_font_feedback",
    LUATEX,
    0,
    "<p>ABCDE FG10.0ptIJKLM N\nO0 P</p>",
    "",
  ),
];

/// Every row converts with its exact Error count (0 Warnings) and its whole paragraph; an Error row names TeX's
/// message. Base (2b854511e8c): "A B C …" spaces, `\pdffontattr` undefined, `\pdffontsize` empty ("Missing number"),
/// no colour stack or font identifier Error, "O0pages P".
#[test]
fn batchpdfprim_pdftex_operand_readers() {
  let mut failures = Vec::new();
  for &(name, preload, errors, paragraph, message) in ROWS {
    let tex = std::fs::read_to_string(format!("{REPROS}{name}.tex")).expect("read repro");
    let (log, xml) = convert_with(&tex, preload);
    let counts = (error_count(&log), warning_count(&log));
    if counts != (errors, 0) || !xml.contains(paragraph) || !log.contains(message) {
      failures.push(format!(
        "{name}: (errors, warnings) = {counts:?}\n{log}\n{xml}"
      ));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n=====\n"));
}
