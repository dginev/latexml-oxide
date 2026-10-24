//! Red/green guards for batch 79i (complete breakage, continued: user 2026-10-24): a shipped copy of a class read
//! after its binding (OXIDIZED_DESIGN_DIVERGENCES #731) does not take over LaTeXML's `\label`. Each guard converts its
//! repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the element whole.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// A shipped aastex7.cls redefines `\label` to write the label to the `.aux` file (aastex7.cls:635), which pdflatex
/// reads back and LaTeXML never does: the copy's definition does not stand for the document — a name LaTeXML's semantic
/// layer owns, a pool `\let` to its own `\lx@label` (state.rs `owned_by_the_semantic_layer`), and since 79j locked
/// against the files (sect11.rs `\label:locked@files`) — so the document's labels are recorded and its references
/// resolve. Red (int81a): the copy's `\label` stood for the
/// document, every `\label` wrote nothing, and each `\ref` was a missing label. Witnesses 2508.00291, 2510.05388,
/// 2604.27071 (aastex7.cls, aastex701.cls shipped). Repro stand-in-copies/labels/copy_redefines_label.
#[test]
fn a_copy_redefining_label_keeps_the_documents_labels() {
  let (log, xml) = convert_files_with(
    repro!("stand-in-copies/labels/copy_redefines_label.tex"),
    &[("aastex7.cls", repro!("stand-in-copies/labels/aastex7.cls"))],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      "Info:provide:\\label \\label keeps LaTeXML's locked definition: the paper's aastex7.cls redefines it"
    ),
    "{log}"
  );
  assert!(
    xml.contains(r#"<section inlist="toc" labels="LABEL:sec:intro" xml:id="S1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<subsection inlist="toc" labels="LABEL:sec:sub" xml:id="S1.SS1">"#),
    "{xml}"
  );
  assert_element(
    &xml,
    "ref",
    &["labelref=\"LABEL:sec:intro\""],
    r#"<ref labelref="LABEL:sec:intro"/>"#,
  );
}
