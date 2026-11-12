//! Red/green guards for batch DTL1: the two pgfmath parser bugs behind pgfplots' "Dimension too large" (52,200
//! Warnings on 2010.05701, 2205.15714, 2311.08887, 2604.09771, 2311.02367, which pdflatex compiles cleanly). One
//! table of the values pdflatex prints, each row's oracle (pdflatex `!` count and value, Perl's count) in its repro
//! header; the error side ("Dimension too large") is a row of `error_boundary`. Mechanism B's pure-parser values are
//! rows of `pgfmath_golden_tests` (latexml_package).
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One repro: its source, the files beside it, its exact (errors, warnings), whole elements `(tag, attributes,
/// element)` and start tags the output holds.
struct Row {
  name:     &'static str,
  tex:      &'static str,
  files:    &'static [(&'static str, &'static str)],
  counts:   (usize, usize),
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
  // B (safe twin): unary minus binds below `^` (pgf's `neg` 500, pgfmathfunctions.basic.code.tex:46-49), so the
  // Gaussian `exp(-(\y)^2)` stays below 1 and the plot is 1.6cm wide (2311.02367); pdflatex 0 `!`, [-4.0]; Perl 0.
  // Read as (-2)^2: [4.0], 2 "Dimension too large", a 22606.82 px picture.
  Row {
    name:     "pgfmath_neg_binds_below_power",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_neg_binds_below_power.tex"
    ),
    files:    &[],
    counts:   (0, 0),
    elements: &[(
      "para",
      "xml:id=\"p1\"",
      r##"<para xml:id="p1"><p xml:id="p1.1">[-4.0]<picture height="213.15" width="62.24" xml:id="p1.pic1"><svg:svg height="213.15" overflow="visible" version="1.1" viewBox="0 0 62.24 213.15" width="62.24"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,213.15) matrix(1 0 0 -1 0 0) translate(0.23,0) translate(0,106.58)"><svg:path d="M 0.04 -106.3 L 0.18 -95.11 L 0.67 -83.92 L 2.08 -72.73 L 5.47 -61.54 L 12.27 -50.35 L 23.42 -39.16 L 38.02 -27.97 L 52.52 -16.78 L 61.73 -5.59 L 61.73 5.59 L 52.52 16.78 L 38.02 27.97 L 23.42 39.16 L 12.27 50.35 L 5.47 61.54 L 2.08 72.73 L 0.67 83.92 L 0.18 95.11 L 0.04 106.3" style="fill:none"/></svg:g></svg:svg></picture></p></para>"##,
    )],
    holds:    &[],
  },
  // A (value side, DTL1e): a box named by an expl3 register, `\wd\l_my_box*2`, read by `try_cs_register`'s
  // expl3-name rule (revDTL1b S2); pdflatex 0 `!`, A[80.0]. Read as `\l` then box 0: A[30.0], 1 Warning.
  Row {
    name:     "pgfmath_box_named_by_expl3",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_box_named_by_expl3.tex"
    ),
    files:    &[],
    counts:   (0, 0),
    elements: &[(
      "para",
      "xml:id=\"p1\"",
      r#"<para xml:id="p1"><p xml:id="p1.1">A[80.0]</p></para>"#,
    )],
    holds:    &[],
  },
  // The tokens-to-text hand-off (DTL1e): a control word's skipped space is kept before a letter, so `\mydim r` and
  // `\wd\mybox r` read the register and the box, then `r` (revDTL1b P1); pdflatex 0 `!`. Welded to `\mydimr`: [0.0]
  // three times, silently.
  Row {
    name:     "pgfmath_space_after_control_word",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_space_after_control_word.tex"
    ),
    files:    &[],
    counts:   (0, 0),
    elements: &[(
      "para",
      "xml:id=\"p1\"",
      r#"<para xml:id="p1"><p xml:id="p1.1">[114.59156][2291.83118][229.18312]</p></para>"#,
    )],
    holds:    &[],
  },
  // One parse for `\pgfmathparse` and `\pgfmathsetmacro` (DTL1f, revDTL1e B1): pdflatex [114.59155] on both sides
  // (our last digit is f64 arithmetic, pgf's TeX's fixed-point dimens); read apart, [0.0].
  // The `\widthof` twins err in TeX: `pgfmath_calc_names_warn.tex`, a Warning each (revDTL1f S1, EB1).
  Row {
    name:     "pgfmath_setmacro_parses_as_pgfmathparse",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_setmacro_parses_as_pgfmathparse.tex"
    ),
    files:    &[],
    counts:   (0, 0),
    elements: &[(
      "para",
      "xml:id=\"p1\"",
      r#"<para xml:id="p1"><p xml:id="p1.1">[114.59156][114.59156]</p></para>"#,
    )],
    holds:    &[],
  },
  // pgf expands, then lets calc's names (revDTL1f B1): a user's `\ratio`/`\real` at every entry point, the `+` path
  // unparsed; the pdflatex values. Letting first: DTL1f [0.0][1][0.0pt][0][0.0] /[345.0pt][345.0pt]… (0 E); Perl
  // [0][1][0.0pt][0][0] (4 E) /[345.0pt][345.0pt]… (4 E).
  Row {
    name:     "pgfmath_expands_before_calc_lets",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_expands_before_calc_lets.tex"
    ),
    files:    &[],
    counts:   (0, 0),
    elements: &[
      (
        "para",
        "xml:id=\"p1\"",
        r#"<para xml:id="p1"><p xml:id="p1.1">[5.0][4.0][5.0pt][5][5.0]</p></para>"#,
      ),
      (
        "para",
        "xml:id=\"p2\"",
        r#"<para xml:id="p2"><p xml:id="p2.1">[172.5pt][177.5pt][5.0pt][0.5pt]</p></para>"#,
      ),
    ],
    holds:    &[],
  },
  // Every parse sets both units flags globally (revDTL1f P1): the pdflatex flags; DTL1f [N][Y][N][Y] [N][Y][N].
  Row {
    name:     "pgfmath_parse_sets_units_flags_globally",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_parse_sets_units_flags_globally.tex"
    ),
    files:    &[],
    counts:   (0, 0),
    elements: &[
      (
        "para",
        "xml:id=\"p1\"",
        r#"<para xml:id="p1"><p xml:id="p1.1">[Y][N][Y][N]</p></para>"#,
      ),
      (
        "para",
        "xml:id=\"p2\"",
        r#"<para xml:id="p2"><p xml:id="p2.1">[Y][N][Y]</p></para>"#,
      ),
    ],
    holds:    &[],
  },
];

/// Every row converted once, in-process: exact counts, the whole elements.
#[test]
fn batchdtl1_pgfmath_rows() {
  for row in ROWS {
    let (log, xml) = convert_files_with(row.tex, row.files, Some("ar5iv.sty"));
    let name = row.name;
    assert_eq!(error_count(&log), row.counts.0, "{name}: errors\n{log}");
    assert_eq!(warning_count(&log), row.counts.1, "{name}: warnings\n{log}");
    for (tag, attrs, element) in row.elements {
      assert_element(&xml, tag, &[attrs], element);
    }
    for start in row.holds {
      assert!(xml.contains(start), "{name}: no {start:?}\n{xml}");
    }
  }
}
