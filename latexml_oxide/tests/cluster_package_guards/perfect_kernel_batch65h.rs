//! Red/green guards for batch 65h, run 340's Error→Fatal regressions (287 papers vs run 338; 107 of them a table read
//! outside any alignment): a package's `\longtable` that calls longtable.sty's `\LT@array` (aalongtable.sty), pasj00's
//! own `{longtable}` when the paper lacks the class, and the cascade of misplaced alignment commands, which 65b made
//! one Error each and so a document-wide `Fatal:TooManyErrors`. Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// The `(log, XML)` of `tex` under ar5iv, with exactly `errors` errors and `warnings` warnings and no Fatal.
fn convert_counted(tex: &str, errors: usize, warnings: usize) -> (String, String) {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), errors, "{log}");
  assert_eq!(warning_count(&log), warnings, "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  (log, xml)
}

/// aalongtable.sty (EDP Sciences 2004, shipped by 81 of the 107 papers) redefines `\longtable` as longtable.sty
/// structures it, `\par\onecolumn\clearpage\begingroup\@ifnextchar[\LT@array{\LT@array[x]}`: the binding's
/// `\LT@array` builds the table (64ze: 4 errors, 65f: 5, then a misplaced-alignment cascade to Fatal in 0707.2342,
/// 0708.2676, 0804.2184, 1102.5033, 1908.10340). Repro class-bindings/aalongtable_lt_array.
#[test]
fn package_longtable_calls_lt_array() {
  let (_, xml) = convert_counted(
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/aalongtable_lt_array.tex"),
    0,
    0,
  );
  assert_element(
    &xml,
    "table",
    &[],
    r#"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><caption><tag close=": ">Table 1</tag>Cap</caption><toccaption><tag close=" ">1</tag>Cap</toccaption><tabular xml:id="S0.T1.1"><tr xml:id="S0.T1.1.1"><td align="left" border="t" xml:id="S0.T1.1.1.1">a</td><td align="left" border="t" xml:id="S0.T1.1.1.2">b</td></tr></tabular></table>"#,
  );
}

/// pasj00.cls defines its own `{longtable}` with longtable.sty's interface (pasj00.cls:884-916, 1241-1244); a paper
/// without the class (OmniBus) keeps it through the longtable binding (astro-ph0407419, 0704.1713, 0705.1365,
/// 1108.4491: `{longtable}` undefined, 65f 101 errors + Fatal), with the class's `\caption`/`\label`: `\noalign`s that
/// gobble a following `\\` if there is one (pasj00.cls:1245-1270), so the head row after `\caption{…}\label{…}`
/// and `\hline` is the head (65h r1 lost it as the caption's row: astro-ph0701701's "Trigger" 28 times at 64ze, 0
/// at r1).
/// Repro class-bindings/pasj00_unshipped_longtable.
#[test]
fn pasj00_without_its_class_keeps_longtable() {
  let (_, xml) = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/pasj00_unshipped_longtable.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "table",
    &[],
    r#"<table inlist="lot" labels="LABEL:t:det" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><caption><tag close=": ">Table 1</tag>Detections</caption><toccaption><tag close=" ">1</tag>Detections</toccaption><tabular xml:id="S0.T1.1"><thead><tr xml:id="S0.T1.1.1"><td align="left" border="t" thead="column" xml:id="S0.T1.1.1.1">Name</td><td align="right" border="t" thead="column" xml:id="S0.T1.1.1.2">Flux</td></tr></thead><tbody><tr xml:id="S0.T1.1.2"><td align="left" border="t" xml:id="S0.T1.1.2.1">a</td><td align="right" border="t" xml:id="S0.T1.1.2.2">1</td></tr><tr xml:id="S0.T1.1.3"><td align="left" border="b" xml:id="S0.T1.1.3.1">b</td><td align="right" border="b" xml:id="S0.T1.1.3.2">2</td></tr></tbody></tabular></table>"#,
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T2""#],
    r#"<table inlist="lot" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><caption><tag close=": ">Table 2</tag>Limits</caption><toccaption><tag close=" ">2</tag>Limits</toccaption><tabular xml:id="S0.T2.1"><thead><tr xml:id="S0.T2.1.1"><td align="left" border="t" thead="column" xml:id="S0.T2.1.1.1">Name</td><td align="right" border="t" thead="column" xml:id="S0.T2.1.1.2">Limit</td></tr></thead><tbody><tr xml:id="S0.T2.1.2"><td align="left" border="t" xml:id="S0.T2.1.2.1">c</td><td align="right" border="t" xml:id="S0.T2.1.2.2">3</td></tr></tbody></tabular></table>"#,
  );
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">Table\u{a0}<ref labelref=\"LABEL:t:det\"/>.</p>",
  );
}

/// longtable.sty's caption is one row ended by `\\`; one without it before `\hline` and the head row (pasj00's idiom
/// under longtable.sty) kept the caption's row-drop (OXIDIZED_DESIGN_DIVERGENCES #103) to the next `\\`, losing the
/// head row (64ze/65f/65h r1). The `&` after the caption shows the row holds more than the caption: it is kept, as
/// pdflatex typesets it (3 errors there, 1 here: the misplaced `\hline`). Repro
/// alignment-bindings/longtable_caption_without_cr_keeps_the_next_row.
#[test]
fn longtable_caption_row_without_its_cr_keeps_the_next_row() {
  let (log, xml) = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/longtable_caption_without_cr_keeps_the_next_row.tex"
    ),
    1,
    0,
  );
  assert!(
    log.contains("Error:unexpected:\\noalign \\noalign cannot be used here\n"),
    "{log}"
  );
  assert_element(
    &xml,
    "table",
    &[],
    r#"<table inlist="lot" labels="LABEL:t" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><caption><tag close=": ">Table 1</tag>Cap</caption><toccaption><tag close=" ">1</tag>Cap</toccaption><tabular xml:id="S0.T1.1"><thead><tr xml:id="S0.T1.1.1"><td align="left" thead="column" xml:id="S0.T1.1.1.1">Head1</td><td align="left" thead="column" xml:id="S0.T1.1.1.2">Head2</td></tr></thead><tbody><tr xml:id="S0.T1.1.2"><td align="left" border="t" xml:id="S0.T1.1.2.1">a</td><td align="left" border="t" xml:id="S0.T1.1.2.2">1</td></tr></tbody></tabular></table>"#,
  );
}

/// A table read outside any alignment (`{longtable}` undefined): the first misplaced `\noalign` is the Error, the
/// 100 misplaced `\noalign`/`\omit`/`\span` and the stray `&` after it in that group are its cascade, Warnings (Perl
/// silences them with `\let`s to `\relax`, TeX_Tables.pool:120-135; 65f: each an Error, 101 + Fatal:TooManyErrors),
/// and the tabular after it spans. Repro kernel-alignment/misplaced_alignment_cascade_is_one_error.
#[test]
fn misplaced_alignment_cascade_is_one_error() {
  let (log, xml) = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/misplaced_alignment_cascade_is_one_error.tex"
    ),
    2,
    101,
  );
  for (count, line) in [
    (
      1,
      "Error:undefined:{longtable} The environment {longtable} is not defined.",
    ),
    (
      1,
      "Error:unexpected:\\noalign \\noalign cannot be used here\n",
    ),
    (
      25,
      "Warning:unexpected:\\noalign \\noalign cannot be used here (in the cascade of a misplaced alignment command \
       above)",
    ),
    (
      50,
      "Warning:unexpected:\\omit \\omit cannot be used here (in the cascade of a misplaced alignment command above)",
    ),
    (
      25,
      "Warning:unexpected:\\span \\span cannot be used here (in the cascade of a misplaced alignment command above)",
    ),
    (
      1,
      "Warning:unexpected:& Stray alignment \"&\" (in the cascade of a misplaced alignment command above)",
    ),
  ] {
    assert_eq!(log.matches(line).count(), count, "{line:?}\n{log}");
  }
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="p2.1"><thead><tr xml:id="p2.1.1"><td align="center" border="l r" colspan="2" thead="column" xml:id="p2.1.1.1">AB</td><td align="right" border="r" thead="column" xml:id="p2.1.1.2">C</td></tr></thead><tbody><tr xml:id="p2.1.2"><td align="left" border="l r" xml:id="p2.1.2.1">d</td><td align="center" border="r" xml:id="p2.1.2.2">e</td><td align="right" border="r" xml:id="p2.1.2.3">f</td></tr></tbody></tabular>"#,
  );
}

/// The cascade is the group's: a misplaced command in the next group is an Error again, whether the group is braced,
/// a `\begingroup…\endgroup` or an environment, and the preamble's ends at `\begin{document}` (65h r1: the preamble's
/// ran through the body, 1 error + 8 warnings); pdflatex 9 errors, Perl's `\let`s end with their group too. Repro
/// kernel-alignment/misplaced_alignment_cascade_ends_with_its_group.
#[test]
fn misplaced_alignment_cascade_ends_with_its_group() {
  let (log, xml) = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/misplaced_alignment_cascade_ends_with_its_group.tex"
    ),
    6,
    3,
  );
  assert_eq!(
    log
      .matches("Error:unexpected:\\noalign \\noalign cannot be used here\n")
      .count(),
    6,
    "{log}"
  );
  for (id, p) in [
    (
      r#"xml:id="p1.1""#,
      r#"<p xml:id="p1.1">one two  three four</p>"#,
    ),
    (r#"xml:id="p2.1""#, r#"<p xml:id="p2.1">five six</p>"#),
    (
      r#"xml:id="p3.1""#,
      r#"<p align="center" xml:id="p3.1">seven eight</p>"#,
    ),
    (r#"xml:id="p4.1""#, r#"<p xml:id="p4.1">nine ten</p>"#),
  ] {
    assert_element(&xml, "p", &[id], p);
  }
}
