//! Red/green guards for batch 67v: the arXiv profile interprets every class no binding covers raw (`rawclasses`),
//! a class in TeX Live as well as one the paper ships (OXIDIZED_DESIGN_DIVERGENCES #444). Run 353: 3,362 papers'
//! TeX Live classes (paper, ptptex, nature, jpsj2, apa6, jacow, smfart, artikel1, …) fell to OmniBus's guesses;
//! interpreted raw, error-free 1,485 → 3,079. Binding routes keep coming first:
//! `perfect_kernel_batch61::a_shipped_class_with_an_alternate_binding_keeps_it`.
use super::perfect_kernel_batch46::{error_count, warning_count};

/// A TeX Live class without a binding (ntgclass's artikel1.cls:204 `\newcommand*\TitleFont{\bfseries}`) is read raw
/// under `ar5iv.sty`, so its commands are defined; before 67v it fell to OmniBus ("Can't find binding for class
/// artikel1", `\TitleFont` undefined). pdflatex: "Hello world." with "Hello" bold.
#[test]
fn a_tex_live_class_without_a_binding_is_interpreted() {
  let (log, xml) = latexml::util::test::convert_with(
    "\\documentclass{artikel1}\n\\begin{document}\n{\\TitleFont Hello} world.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("OmniBus"), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Hello</text> world.</p>"#,
  );
}
