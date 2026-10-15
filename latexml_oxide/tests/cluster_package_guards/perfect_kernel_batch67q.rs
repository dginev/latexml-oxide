//! Red/green guards for batch 67q: sandbox-2605 regressions of the 1a95291ef6 baseline (run 354). mathpazo declares
//! its `upright` symbol font (mathpazo.sty:63), which flexisym's mathpazo.sym reads. The `\path` of the arXiv
//! hyperref fallback is in `perfect_kernel_batch67a`.
use super::perfect_kernel_batch46::{error_count, warning_count};

/// `\symupright` exists after `\usepackage{mathpazo}` (2605.15826, 2605.30950).
#[test]
fn mathpazo_declares_the_upright_symbol_font() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/mathpazo_declares_upright_symbol_font.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">Upright font: declared.</p>"#,
  );
}

/// hyperref keeps the original `\maketitle` as `\HyOrg@maketitle`; a class that restores it gets the title
/// (pittetd.cls, 1509.06375).
#[test]
fn hyperref_keeps_the_original_maketitle() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/hyperref_hyorg_maketitle_kept.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(&xml, "title", &[], "<title>A Thesis</title>");
}
