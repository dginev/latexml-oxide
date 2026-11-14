//! Red/green guards for batch 79t (complete breakage, continued: user 2026-10-24): AASTeX 7's `{longdeluxetable}`,
//! xcolor's name sets after an earlier `color`, pdfTeX's interword-space primitives. Each guard converts its repro
//! under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `\begin{longdeluxetable}` (aastex701.cls:11516/:11614, the long table `\deluxetable` turns to after
/// `\startlongtable`): aas_support_sty.rs `\longdeluxetable` is that table, so its caption and `\label` are kept and it
/// is numbered among the others. Red (int89a): "The environment {longdeluxetable} is not defined", the caption and
/// label lost. Witness 2608.15711. Repro class-bindings/aastex_longdeluxetable.
#[test]
fn a_long_deluxetable_keeps_its_caption_and_label() {
  let (log, xml) = convert_files_with(
    repro!("class-bindings/aastex_longdeluxetable.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<table ").count(), 3, "{xml}");
  for label in ["tab:a", "tab:b", "tab:c"] {
    assert_eq!(
      xml.matches(&format!("labels=\"LABEL:{label}\"")).count(),
      1,
      "{label}: {xml}"
    );
  }
  assert!(xml.contains("Second table."), "{xml}");
  assert!(xml.contains(">Table 2<"), "{xml}");
}

/// A global `dvipsnames` with `color` loaded first and xcolor after (lipics-v2021's order): xcolor_sty.rs reads
/// `dvipsnam.def` again for its own names (xcolor.sty:193 `\XC@declarenames`, an `\input`), so `RoyalBlue` is found.
/// Red (int89a): "Can't find color named 'RoyalBlue'; assuming Black". Witness 2407.14159. Repro
/// package-bindings/xcolor_dvipsnames_after_color.
#[test]
fn xcolor_reads_the_dvips_names_after_color_has() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/xcolor_dvipsnames_after_color.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r##"color="#0080FF""##), "{xml}");
  assert!(xml.contains("Blue words."), "{xml}");
}

/// pdfTeX's `\pdfinterwordspaceon`/`\pdfinterwordspaceoff`/`\pdffakespace` (pdftex.rs) typeset nothing. Red (int89a):
/// each undefined. Witness 2205.10368. Repro expansion-primitives/pdftex_interwordspace_primitives.
#[test]
fn pdftex_interword_space_primitives_typeset_nothing() {
  let (log, xml) = convert_files_with(
    repro!("expansion-primitives/pdftex_interwordspace_primitives.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Before fakespaces after."), "{xml}");
}
