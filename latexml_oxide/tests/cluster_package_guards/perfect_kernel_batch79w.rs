//! Red/green guards for batch 79w (complete breakage, continued: user 2026-10-24): `\valign` sets its columns side by
//! side; a pgfkeys key that expands to one brace group names the key inside it. Each guard converts its repro under
//! `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// tex_tables.rs `\valign` reads its body as `\halign`'s, each entry in a `\vbox`, and turns the lines read into the
/// table's columns (`Alignment::transpose`): `a1&a2\cr b1&b2&b3\cr` is two columns side by side, a1 beside b1 in the
/// first row and b3 alone in the third; a column's `\par` keeps its two paragraphs; and an environment's
/// `\valign\bgroup…\cr` closed by its end's `\crcr\egroup` keeps its panels, the figure's caption, label and reference
/// and the text after it. Red (int92a): "File ended while scanning text of \valign", every entry, the caption, the
/// label and "After" lost. Witness 2502.06786. Repro kernel-alignment/valign_columns_side_by_side.
#[test]
fn valign_sets_its_columns_side_by_side() {
  let (log, xml) = convert_files_with(
    repro!("kernel-alignment/valign_columns_side_by_side.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<tabular ").count(), 3, "{xml}");
  let rows: Vec<&str> = xml.split("<tr ").skip(1).collect();
  let in_row = |row: usize, text: &str| rows[row].contains(&format!(">{text}</p>"));
  assert!(in_row(0, "a1") && in_row(0, "b1"), "{xml}");
  assert!(in_row(1, "a2") && in_row(1, "b2"), "{xml}");
  assert!(in_row(2, "b3") && !in_row(2, "a2"), "{xml}");
  assert!(
    in_row(3, "First column top.") && in_row(3, "Still first column."),
    "{xml}"
  );
  assert!(in_row(3, "Second column."), "{xml}");
  assert!(
    in_row(4, "Panel A text.") && in_row(4, "Panel B text."),
    "{xml}"
  );
  assert!(xml.contains(r#"labels="LABEL:fig:panels""#), "{xml}");
  assert!(xml.contains("The two panels."), "{xml}");
  assert!(
    xml.contains(r#"<ref labelref="LABEL:fig:panels"/>"#),
    "{xml}"
  );
  assert!(
    xml.contains("between.") && xml.contains("After, see Figure"),
    "{xml}"
  );
}

/// pgfkeys_code_tex.rs `dispatch_item` takes the expanded key without a leading `/` as `\pgfkeys@addpath#1\relax` does
/// (pgfkeys.code.tex:540-543), a delimited argument that loses one brace group: `\def\RelayedType{{node}}` …
/// `\node[\RelayedType, draw=black]` uses the style `\tikzstyle{node}` defined, both vertices drawn as its circle. Red
/// (int92a): "I do not know the key '/tikz/node'" for each, no circle drawn. Witness 2110.08325 (`\drawGridVertex`,
/// reached once 79v found its `\input` files). Repro tikz-pgf/pgfkeys_braced_key_in_a_macro.
#[test]
fn a_braced_key_in_a_macro_names_its_style() {
  let (log, xml) = convert_files_with(
    repro!("tikz-pgf/pgfkeys_braced_key_in_a_macro.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches(">v<").count(), 2, "{xml}");
  let circles = xml
    .split("<svg:path ")
    .skip(1)
    .filter(|path| {
      path
        .split('>')
        .next()
        .is_some_and(|attrs| attrs.contains(" C "))
    })
    .count();
  assert_eq!(circles, 2, "{xml}");
  assert!(xml.contains("Done."), "{xml}");
}
