//! Red/green guards for batch 79j (complete breakage, continued: user 2026-10-24): `\label` is locked against the
//! class and package files, as the kernel minipage is, so a raw class's own `\label` — the kernel's write of
//! `\newlabel` to the `.aux`, which LaTeXML never reads back — does not leave the document's labels unrecorded. Each
//! guard converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the element whole.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// A shipped class with no binding, read raw, redefines `\label` to write the label to the `.aux` (SIAM's
/// siamltex-derived CMSLATEX.CLS:736, siuro210301.cls; jasatex.cls, myajour.cls): the redefinition is refused, as
/// for every name locked against the files (sect11.rs `\label:locked@files`), and the section and equation carry
/// their labels. Red (int82a): the raw class's `\label` stood, no label was recorded, and each reference was a
/// missing label. Witnesses 1607.06796, 2412.10948, 1504.01662, 1009.5642, math-ph0209020. Repro
/// package-bindings/raw_class_redefines_label.
#[test]
fn a_raw_class_redefining_label_keeps_the_documents_labels() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/raw_class_redefines_label.tex"),
    &[(
      "rawlabelclass.cls",
      repro!("package-bindings/rawlabelclass.cls"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains("Info:ignore:\\label Ignoring redefinition of \\label"),
    "{log}"
  );
  assert!(
    xml.contains(r#"<section inlist="toc" labels="LABEL:sec:intro" xml:id="S1">"#),
    "{xml}"
  );
  assert_element(
    &xml,
    "ref",
    &["labelref=\"LABEL:eq:one\""],
    r#"<ref labelref="LABEL:eq:one"/>"#,
  );
  assert!(
    xml.contains(r#"<equation labels="LABEL:eq:one" xml:id="S1.E1">"#),
    "{xml}"
  );
}
