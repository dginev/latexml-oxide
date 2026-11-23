//! Red/green guards for batch PDFPRIM: pdfTeX's operand readers as pdftex.web reads them — one optional space after
//! `reserveobjnum`, a dest type and `newwindow` (`scan_action`'s order, `struct`), `\pdffontattr`, the font queries'
//! number and size, and the Errors TeX gives for a missing colour stack action, a missing font identifier, pdfTeX's
//! `pdf_error`s in its readers and the null font; `\DeclareFixedFont` as a font identifier. Each row is a
//! pdflatex/lualatex-checked repro; every Error row has a 0-error twin.
use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

const LUATEX: Option<&str> = Some("[rawstyles,rawclasses,luatex]latexml.sty");
const REPROS: &str = concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/../tools/perfect_kernel/repros/"
);

/// A row: (repro, preload, errors, paragraph, the Error messages in log order).
type Row = (
  &'static str,
  Option<&'static str>,
  usize,
  &'static str,
  &'static [&'static str],
);

/// The rows, one per distinct code path: the pdfTeX operand readers (the 0-error twin of the reader Errors), the colour
/// stack Error, the font queries (twin of the null font's), the reader `pdf_error`s — a missing dest type, a missing
/// identifier (`read_id_spec`, `scan_action`), a missing action type, the value checks —, the null font's "invalid font
/// identifier" (`read_pdf_font_ident`, after `read_font_ident`'s "Missing font identifier"), `read_font_ident`'s Error,
/// `\DeclareFixedFont`'s identifier (`\fontname` through `font_id`), and LuaTeX's `\pdfextension`/`\pdffeedback`
/// path through the same readers.
const ROWS: &[Row] = &[
  (
    "backend-persona/pdftex_operands_optional_space",
    None,
    0,
    "<p>ABCDEFGHIJ KLMNOPQ R STUV</p>",
    &[],
  ),
  (
    "backend-persona/pdfcolorstack_action_missing",
    None,
    1,
    "<p>AxBC D</p>",
    &["Error:expected:\\pdfcolorstack Color stack action is missing"],
  ),
  (
    "backend-persona/pdffont_feedback_numbers",
    None,
    0,
    "<p>AB12.0ptC10.0ptDEFXHIJ12.0ptK</p>",
    &[],
  ),
  (
    "backend-persona/pdfdest_destination_type_missing",
    None,
    1,
    "<p>AB</p>",
    &["Error:expected:\\pdfdest pdfTeX error (ext1): destination type missing"],
  ),
  (
    "backend-persona/pdftex_identifier_type_missing",
    None,
    4,
    "<p>ABd fit CxDEFG</p>",
    &[
      "Error:expected:\\pdfstartlink pdfTeX error (ext1): identifier type missing",
      "Error:expected:\\pdfdest pdfTeX error (ext1): identifier type missing",
      "Error:expected:\\pdfthread pdfTeX error (ext1): identifier type missing",
      "Error:expected:\\pdfstartlink pdfTeX error (ext1): identifier type missing",
    ],
  ),
  (
    "backend-persona/pdftex_action_type_missing",
    None,
    3,
    "<p>AxBCDxEF</p>",
    &[
      "Error:expected:\\pdfstartlink pdfTeX error (ext1): action type missing",
      "Error:expected:\\pdfoutline pdfTeX error (ext1): action type missing",
      "Error:expected:\\pdfcatalog pdfTeX error (ext1): action type missing",
    ],
  ),
  (
    "backend-persona/pdftex_action_value_errors",
    None,
    7,
    "<p>ABCDEFGHIJKLMN</p>",
    &[
      "Error:expected:\\pdfdest pdfTeX error (ext1): num identifier must be positive",
      "Error:expected:\\pdfdest pdfTeX error (ext1): struct identifier must be positive",
      "Error:expected:\\pdfstartlink pdfTeX error (ext1): page number must be positive",
      "Error:expected:\\pdfstartlink pdfTeX error (ext1): only GoTo action can be used with `page'",
      "Error:expected:\\pdfstartlink pdfTeX error (ext1): only GoTo action can be used with `struct'",
      "Error:expected:\\pdfstartlink pdfTeX error (ext1): `goto' option cannot be used with both `file' and `num'",
      "Error:expected:\\pdfstartlink pdfTeX error (ext1): `newwindow'/`nonewwindow' must be used with `goto' and `file' option",
    ],
  ),
  (
    "backend-persona/pdffont_invalid_font_identifier",
    None,
    5,
    "<p>A0.0ptB0.0ptxCDE</p>",
    &[
      "Error:invalid:\\nullfont pdfTeX error (font): invalid font identifier",
      "Error:expected:font identifier Missing font identifier",
      "Error:invalid:\\nullfont pdfTeX error (font): invalid font identifier",
      "Error:invalid:\\nullfont pdfTeX error (font): invalid font identifier",
      "Error:invalid:\\nullfont pdfTeX error (font): invalid font identifier",
    ],
  ),
  (
    "fonts-nfss/missing_font_identifier",
    None,
    2,
    "<p>A0.0ptxBnullfont3C</p>",
    &[
      "Error:expected:font identifier Missing font identifier",
      "Error:expected:font identifier Missing font identifier",
    ],
  ),
  (
    "fonts-nfss/declarefixedfont_font_identifier",
    None,
    0,
    "<p>Acmr12B11.74988ptC45D-1E<text fontsize=\"120%\">x</text>F</p>",
    &[],
  ),
  (
    "luatex-profile/pdfextension_optional_space_font_feedback",
    LUATEX,
    0,
    "<p>ABCDE FG10.0ptIJKLM N\nOlastximagepages Plastximage Qlastxform R</p>",
    &[],
  ),
];

/// Whether `messages` occur in `log` in this order.
fn in_order(log: &str, messages: &[&str]) -> bool {
  let mut rest = log;
  for message in messages {
    let Some(at) = rest.find(message) else {
      return false;
    };
    rest = &rest[at + message.len()..];
  }
  true
}

/// Every row converts with its exact Error count (0 Warnings), its whole paragraph and TeX's messages in TeX's order.
/// Base (2b854511e8c): "A B C …" spaces, `\pdffontattr` undefined, `\pdffontsize` empty ("Missing number"), no colour
/// stack or font identifier Error, "O0pages P". Round 0 (a7b7a73375c): no reader `pdf_error`, no null-font Error from
/// `\pdffontattr`/`\pdfincludechars`, `\DeclareFixedFont`'s `\relax` "Missing font identifier" ×4, "O0 P0 Q0 R".
#[test]
fn batchpdfprim_pdftex_operand_readers() {
  let mut failures = Vec::new();
  for &(name, preload, errors, paragraph, messages) in ROWS {
    let tex = std::fs::read_to_string(format!("{REPROS}{name}.tex")).expect("read repro");
    let (log, xml) = convert_with(&tex, preload);
    let counts = (error_count(&log), warning_count(&log));
    if counts != (errors, 0) || !xml.contains(paragraph) || !in_order(&log, messages) {
      failures.push(format!(
        "{name}: (errors, warnings) = {counts:?}\n{log}\n{xml}"
      ));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n=====\n"));
}
