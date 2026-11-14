//! mhchem's formula output as chemistry math (issue 824; OXIDIZED_DESIGN_DIVERGENCES #1039): mhchem's own parser reads
//! `\ce`, and its output hook (`\__mhchem_output_begin_cf:`, mhchem.sty:1424) turns each `\__mhchem_output_coreFive:nnnnnnn`
//! unit (mhchem.sty:2774) into element atoms with their isotope, count and charge scripts. Repros
//! `tools/perfect_kernel/repros/package-bindings/mhchem_formula_*.tex`.
use latexml::util::test::{convert_bytes_files_then, kpse_has};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// A repro: its name, its source, every `<math>` of its HTML page in order.
type Row = (&'static str, &'static [u8], &'static [&'static str]);

const ROWS: &[Row] = &[
  (
    "mhchem_formula_units",
    include_bytes!(
      "../../../tools/perfect_kernel/repros/package-bindings/mhchem_formula_units.tex"
    ),
    &[
      "<math id=\"p1.m1\" class=\"ltx_Math\" alttext=\"{}^{\\mathrm{95}}\\mathord{\\mathrm{Mo}}\" display=\"inline\" intent=\":literal\"><mmultiscripts><mi>Mo</mi><mprescripts></mprescripts><mrow></mrow><mn>95</mn></mmultiscripts></math>",
      "<math id=\"p2.m1\" class=\"ltx_Math\" alttext=\"\\mathord{{}^{\\mathrm{95}}\\mathord{\\mathrm{Mo}}\\mathord{\\mathrm{O}}_{\\mathrm{4}}}^{\\mathrm{2-}}\" display=\"inline\" intent=\":literal\"><msup><mrow><mmultiscripts><mi>Mo</mi><mprescripts></mprescripts><mrow></mrow><mn>95</mn></mmultiscripts><mo>\u{2062}</mo><msub><mi mathvariant=\"normal\">O</mi><mn>4</mn></msub></mrow><mrow><mn>2</mn><mo>−</mo></mrow></msup></math>",
      "<math id=\"p3.m1\" class=\"ltx_Math\" alttext=\"\\mathord{\\mathrm{H}}_{\\mathrm{2}}\\mathord{\\mathrm{O}}\" display=\"inline\" intent=\":literal\"><mrow><msub><mi mathvariant=\"normal\">H</mi><mn>2</mn></msub><mo>\u{2062}</mo><mi mathvariant=\"normal\">O</mi></mrow></math>",
      "<math id=\"p4.m1\" class=\"ltx_Math\" alttext=\"\\mathord{\\mathord{\\mathrm{S}}\\mathord{\\mathrm{O}}_{\\mathrm{4}}}^{\\mathrm{2-}}\" display=\"inline\" intent=\":literal\"><msup><mrow><mi mathvariant=\"normal\">S</mi><mo>\u{2062}</mo><msub><mi mathvariant=\"normal\">O</mi><mn>4</mn></msub></mrow><mrow><mn>2</mn><mo>−</mo></mrow></msup></math>",
      "<math id=\"p5.m1\" class=\"ltx_Math\" alttext=\"{}^{\\mathrm{227}}_{\\mathrm{90}}\\mathord{\\mathrm{Th}}^{\\mathrm{+}}\" display=\"inline\" intent=\":literal\"><mmultiscripts><mi>Th</mi><mrow></mrow><mo>+</mo><mprescripts></mprescripts><mn>90</mn><mn>227</mn></mmultiscripts></math>",
      "<math id=\"p6.m1\" class=\"ltx_Math\" alttext=\"\\mathord{\\mathord{\\mathrm{Fe}}\\mathrm{(}\\mathord{\\mathrm{C}}\\mathord{\\mathrm{N}}\\mathrm{)}_{\\mathrm{6}}}^{\\mathrm{3-}}\" display=\"inline\" intent=\":literal\"><msup><mrow><mi>Fe</mi><mo>\u{2062}</mo><msub><mrow><mo stretchy=\"false\">(</mo><mrow><mi mathvariant=\"normal\">C</mi><mo>\u{2062}</mo><mi mathvariant=\"normal\">N</mi></mrow><mo stretchy=\"false\">)</mo></mrow><mn>6</mn></msub></mrow><mrow><mn>3</mn><mo>−</mo></mrow></msup></math>",
      "<math id=\"p7.m1\" class=\"ltx_Math\" alttext=\"x={}\\mathord{\\mathord{\\mathrm{S}}\\mathord{\\mathrm{O}}_{\\mathrm{4}}}^{\\mathrm{2-}}\" display=\"inline\" intent=\":literal\"><mrow><mi>x</mi><mo>=</mo><msup><mrow><mi mathvariant=\"normal\">S</mi><mo>\u{2062}</mo><msub><mi mathvariant=\"normal\">O</mi><mn>4</mn></msub></mrow><mrow><mn>2</mn><mo>−</mo></mrow></msup></mrow></math>",
      "<math id=\"p8.m1\" class=\"ltx_Math\" alttext=\"\\mathord{\\mathord{\\mathrm{O}}\\mathord{\\mathrm{H}}}^{\\mathrm{-}}\\mathrm{(}\\mathord{\\mathrm{H}}_{\\mathrm{2}}\\mathord{\\mathrm{O}}\\mathrm{)}\" display=\"inline\" intent=\":literal\"><mrow><msup><mrow><mi mathvariant=\"normal\">O</mi><mo>\u{2062}</mo><mi mathvariant=\"normal\">H</mi></mrow><mo>−</mo></msup><mo>\u{2062}</mo><mrow><mo stretchy=\"false\">(</mo><mrow><msub><mi mathvariant=\"normal\">H</mi><mn>2</mn></msub><mo>\u{2062}</mo><mi mathvariant=\"normal\">O</mi></mrow><mo stretchy=\"false\">)</mo></mrow></mrow></math>",
    ],
  ),
  (
    "mhchem_formula_text_font",
    include_bytes!(
      "../../../tools/perfect_kernel/repros/package-bindings/mhchem_formula_text_font.tex"
    ),
    &[
      "<math id=\"p1.m1\" class=\"ltx_Math\" alttext=\"\\mathord{\\mathrm{N}}\\mathord{\\mathrm{O}}\" display=\"inline\" intent=\":literal\"><mrow><mi>𝐍</mi><mo>\u{2062}</mo><mi>𝐎</mi></mrow></math>",
      "<math id=\"p1.m2\" class=\"ltx_Math\" alttext=\"\\mathord{\\mathord{\\mathrm{S}}\\mathord{\\mathrm{O}}_{\\mathrm{4}}}^{\\mathrm{2-}}\" display=\"inline\" intent=\":literal\"><msup><mrow><mi>𝖲</mi><mo>\u{2062}</mo><msub><mi>𝖮</mi><mn>𝟦</mn></msub></mrow><mrow><mn>𝟤</mn><mo>−</mo></mrow></msup></math>",
      "<math id=\"p1.m3\" class=\"ltx_Math\" alttext=\"\\mathord{\\mathrm{H}}_{\\mathrm{2}}\\mathord{\\mathrm{O}}\" display=\"inline\" intent=\":literal\"><mrow><msub><mi>H</mi><mn class=\"ltx_mathvariant_italic\" mathvariant=\"italic\">2</mn></msub><mo>\u{2062}</mo><mi>O</mi></mrow></math>",
      "<math id=\"p2.m1\" class=\"ltx_Math\" alttext=\"{}{}\\mathord{\\mathrm{H}}_{\\mathrm{2}}\\mathord{\\mathrm{O}}\" display=\"inline\" intent=\":literal\"><mrow><msub><mi mathvariant=\"normal\">H</mi><mn>2</mn></msub><mo>\u{2062}</mo><mi mathvariant=\"normal\">O</mi></mrow></math>",
    ],
  ),
];

#[test]
fn mhchem_formula_units_are_chemistry() {
  if !kpse_has("mhchem.sty") {
    eprintln!("SKIP: mhchem.sty is not in this TeX Live");
    return;
  }
  for (name, tex, maths) in ROWS {
    let (log, _, (html, post_log)) =
      convert_bytes_files_then(tex, &[], Some("ar5iv.sty"), |xml, source_dir| {
        super::perfect_kernel_batch64zf::post_logged(
          xml,
          source_dir.to_str().expect("a UTF-8 path"),
        )
      });
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}:\n{log}"
    );
    assert_eq!(error_count(&post_log), 0, "{name}:\n{post_log}");
    let found: Vec<&str> = html
      .match_indices("<math ")
      .map(|(at, _)| {
        &html[at..at + html[at..].find("</math>").expect("a closed <math>") + "</math>".len()]
      })
      .collect();
    assert_eq!(&found, maths, "{name}");
  }
}
