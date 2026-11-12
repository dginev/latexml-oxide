//! Red/green guards for batch DTL1: the two pgfmath parser bugs behind pgfplots' "Dimension too large" (52,200
//! Warnings on 2010.05701, 2205.15714, 2311.08887, 2604.09771, 2311.02367, which pdflatex compiles cleanly). One
//! table, both sides of the error boundary: each row names its oracle (pdflatex `!` count and value, Perl's count) in
//! its repro header. Mechanism B's pure-parser values are rows of `pgfmath_golden_tests` (latexml_package).
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One repro: its source, the files beside it, its exact (errors, warnings), the diagnostics counted, whole elements
/// `(tag, attributes, element)` and start tags the output holds.
struct Row {
  name:     &'static str,
  tex:      &'static str,
  files:    &'static [(&'static str, &'static str)],
  counts:   (usize, usize),
  messages: &'static [&'static str],
  elements: &'static [(&'static str, &'static str, &'static str)],
  holds:    &'static [&'static str],
}

const ROWS: &[Row] = &[
  // A (value side): `\wd`/`\ht`/`\dp` of a box named by a register or by digits, scaled, summed, negated — the
  // pdflatex values (Perl pgfmath.code.tex.ltxml:696-701; `\wd0` beyond Perl, OXIDIZED_DESIGN_DIVERGENCES #1004).
  Row {
    name:     "pgfmath_box_dimensions",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_box_dimensions.tex"
    ),
    files:    &[],
    counts:   (0, 0),
    messages: &[],
    elements: &[
      (
        "para",
        "xml:id=\"p1\"",
        r#"<para xml:id="p1"><p xml:id="p1.1">[100.0][7.0][3.0]</p></para>"#,
      ),
      (
        "para",
        "xml:id=\"p2\"",
        r#"<para xml:id="p2"><p xml:id="p2.1">[50.0][4.0][2.0][50.0]</p></para>"#,
      ),
      (
        "para",
        "xml:id=\"p3\"",
        r#"<para xml:id="p3"><p xml:id="p3.1">[200.0][45.0][2.0][-1.0]</p></para>"#,
      ),
      (
        "para",
        "xml:id=\"p4\"",
        r#"<para xml:id="p4"><p xml:id="p4.1">[30.0][25.0]</p></para>"#,
      ),
    ],
    holds:    &[],
  },
  // A (safe twin): tikzscale sizes a pgfplots axis to 5cm by `\wd` of its measuring box (tikzscale.sty:392,
  // 522-524); pdflatex 142.26459pt, 0 `!`. Read as 0 the scale clamped: 2 Errors, 104 "Dimension too large".
  Row {
    name:     "pgfmath_tikzscale_width",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_tikzscale_width.tex"
    ),
    files:    &[(
      "pgfmath_tikzscale_width.tikz",
      include_str!(
        "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_tikzscale_width.tikz"
      ),
    )],
    counts:   (0, 0),
    messages: &[],
    elements: &[(
      "para",
      "xml:id=\"p1\"",
      r#"<para xml:id="p1"><p xml:id="p1.1">[142.26471pt]</p></para>"#,
    )],
    holds:    &[
      r#"<picture height="137.12" width="196.85" xml:id="p2.pic1">"#,
      "matrix(0.77832 0.0 0.0 0.77832 -12.78 -14.33)",
    ],
  },
  // The error side: a dimension TeX cannot hold still reports "Dimension too large" (pdflatex 1 `!`; Perl 0 and
  // 20000.0pt). A Warning today; the out-of-range severity batch makes it an Error.
  Row {
    name:     "pgfmath_dimension_too_large_still_reported",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_dimension_too_large_still_reported.tex"
    ),
    files:    &[],
    counts:   (0, 1),
    messages: &["Dimension too large"],
    elements: &[(
      "para",
      "xml:id=\"p1\"",
      r#"<para xml:id="p1"><p xml:id="p1.1">[16383.99998pt]</p></para>"#,
    )],
    holds:    &[],
  },
];

/// Every row converted once, in-process: exact counts, the diagnostics named, the whole elements.
#[test]
fn batchdtl1_pgfmath_rows() {
  for row in ROWS {
    let (log, xml) = convert_files_with(row.tex, row.files, Some("ar5iv.sty"));
    let name = row.name;
    assert_eq!(error_count(&log), row.counts.0, "{name}: errors\n{log}");
    assert_eq!(warning_count(&log), row.counts.1, "{name}: warnings\n{log}");
    for message in row.messages {
      assert!(
        log
          .lines()
          .any(|l| l.starts_with("Warning:") && l.contains(message)),
        "{name}: no Warning {message:?}\n{log}"
      );
    }
    for (tag, attrs, element) in row.elements {
      assert_element(&xml, tag, &[attrs], element);
    }
    for start in row.holds {
      assert!(xml.contains(start), "{name}: no {start:?}\n{xml}");
    }
  }
}
