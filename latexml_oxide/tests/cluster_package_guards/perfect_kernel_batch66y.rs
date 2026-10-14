//! Red/green guards for batch 66y: the upstream roots of `_`/`^` typed outside math (K1 parked by user ruling
//! 2026-10-14, `agent_reports/2026-10-14_k1_arxiv_pdf_study.md`).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// In a picture `\raisebox` is `\pic@raisebox`; its box is an `\hbox` (latex.ltx `\@argrsbox`), so a `$+$` inside it is
/// a formula of its own and the formula around it stays whole (0902.1321).
#[test]
fn picture_raisebox_reads_its_box_in_text() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/picture_raisebox_reads_its_box_in_text.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<Math mode="inline" tex="4u\raisebox{0.86108pt}{\tiny$+$}2u^{2}""#),
    "{xml}"
  );
  let raised = xml
    .split(r#"<g y="0.9pt">"#)
    .nth(1)
    .and_then(|rest| rest.split("</g>").next())
    .unwrap_or_else(|| panic!("no raised box: {xml}"));
  assert!(
    raised.contains(r#"<Math mode="inline" tex="+" text="+""#),
    "{raised}"
  );
}

/// An `array` outside math is a formula of its own: latex.ltx's `\@arrayclassz` sets every cell as `$\relax\@sharp$`, so
/// its cells are math, in a paragraph as in a tabular cell (1611.02392 `bnfarray`); the text after it stays text.
#[test]
fn array_in_text_has_math_cells() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/array_in_text_has_math_cells.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<Math mode="inline" tex="\begin{array}[t]{ll}x_{1}&amp;y^{2}\\&#10;a&amp;b\end{array}" text="Array[[x _ 1, y ^ 2], [a, b]]""#),
    "{xml}"
  );
  assert!(xml.contains("</Math> after.</p>"), "{xml}");
  assert!(
    xml.contains(r#"<td align="center" xml:id="p2.1.1.1"><Math mode="inline" tex="\begin{array}[t]{l@{~~}l}e_{1}&amp;e_{2}\end{array}""#),
    "{xml}"
  );
  assert!(!xml.contains("<XMText"), "{xml}");
}

/// In math an `array` opens no formula of its own, nested in another array's cell too: the outer formula closes only at
/// its own `$`.
#[test]
fn array_in_math_opens_no_formula() {
  let tex = "\\documentclass{article}\n\\begin{document}\n$\\begin{array}{c} \\begin{array}{c} a_1 \\end{array} \\\\ b \
             \\end{array} + c$ after.\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
  assert_eq!(xml.matches("<XMArray ").count(), 2, "{xml}");
  assert!(xml.contains("</Math> after.</p>"), "{xml}");
}

/// A `[c]` array's `\vcenter` in text is TeX's "Missing $ inserted" (latex.ltx `\@array`); one Error, and the array is
/// set as a formula of its own, the text after it outside.
#[test]
fn centred_array_in_text_is_missing_dollar() {
  let tex = "\\documentclass{article}\n\\begin{document}\nOut \\begin{array}{c} k_1 \\end{array} tail.\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(
    log.matches("Error:expected:$ Missing $ inserted").count(),
    1,
    "{log}"
  );
  assert!(xml.contains("</Math> tail.</p>"), "{xml}");
  assert!(!log.contains("Script _"), "{log}");
}

/// An array built on `\@tabarray` (t-angles.sty:491, ucthesis.cls:626) inside a text array's cell does not close the
/// outer formula: every `\@array` setup resets the close in its own group.
#[test]
fn inner_tabarray_inside_text_array() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\def\\myarr{\\let\\@classz\\@arrayclassz\\@tabarray}\
             \\def\\endmyarr{\\endarray}\n\\makeatother\n\\begin{document}\nOut \\begin{array}[t]{c} \\begin{myarr}{c} \
             k_1 \\end{myarr} \\\\ w_2 \\end{array} tail.\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
  assert_eq!(xml.matches("<XMArray ").count(), 2, "{xml}");
  assert!(xml.contains("</Math> tail.</p>"), "{xml}");
}

/// delarray's `\array` (memoir.cls:5468 loads it) outside math is a formula of its own too, its delimiters inside it.
#[test]
fn delarray_array_in_text() {
  for (preamble, body) in [
    (
      "\\usepackage{delarray}",
      "\\begin{array}[t]{c} k_1 \\end{array}",
    ),
    (
      "\\usepackage{delarray}",
      "\\begin{array}[t]({c}) k_1 \\end{array}",
    ),
  ] {
    let tex = format!(
      "\\documentclass{{article}}\n{preamble}\n\\begin{{document}}\nOut {body} tail.\n\\end{{document}}\n"
    );
    let (log, xml) = convert_counted(&tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{body}: {log}"
    );
    assert_eq!(xml.matches("<Math ").count(), 1, "{body}: {xml}");
    assert!(xml.contains("</Math> tail.</p>"), "{body}: {xml}");
  }
}
