//! Red/green guards for batch REGTAILD (row R REGTAIL: papers an old build converted cleanly that later erred): under
//! amsthm a `\relax` counter argument names no counter; sidecap's floats are the kernel's figure and table; mathtools'
//! paired delimiters are robust commands. Repros under `tools/perfect_kernel/repros/package-bindings/`.
use latexml::util::test::{convert_with, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The `<tags>` of the theorem `id`.
fn theorem_tags(xml: &str, id: &str) -> String {
  let theorem = xml_element(xml, "theorem", &[&format!("xml:id=\"{id}\"")])
    .unwrap_or_else(|| panic!("no theorem {id}\n{xml}"));
  xml_element(&theorem, "tags", &[]).unwrap_or_else(|| panic!("no tags in {theorem}"))
}

fn tags(display: &str, refnum: &str) -> String {
  format!(
    "<tags><tag>{display}</tag><tag role=\"refnum\">{refnum}</tag><tag role=\"typerefnum\">{display}</tag></tags>"
  )
}

/// amsthm.sty:94/:114 `\ifx\relax#2\relax`: under amsthm a `\relax` in the shared-counter or the parent argument names
/// no counter, as thm-kv.sty:401 passes AVT.sty:583's `numberwithin=\ifAVT@notes…\relax\else\relax\fi` (2208.14960,
/// 2309.00854, 2409.18804: 1891 Errors). Red (INT081I): 17 Errors per form, "Theorem 0.1". The kernel's `\@xnthm` has
/// no such test and errs, as pdflatex (6 there; ours at each `\csname the\relax\endcsname`).
#[test]
fn amsthm_relax_counter_argument_names_no_counter() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/amsthm_newtheorem_relax_names_no_counter.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, display, refnum) in [
    ("Thmtheorem1", "Theorem 1", "1"),
    ("Thmlemma1", "Lemma 1", "1"),
    ("S1.Thmclaim1", "Claim 1.1", "1.1"),
  ] {
    assert_eq!(theorem_tags(&xml, id), tags(display, refnum), "{xml}");
  }
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/thmtools_numberwithin_relax_names_no_counter.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, display) in [("Thmremark1", "Remark 1"), ("Thmtheorem1", "Theorem 1")] {
    assert_eq!(theorem_tags(&xml, id), tags(display, "1"), "{xml}");
  }
  let (log, _) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/newtheorem_relax_parent_without_amsthm_errs.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (17, 0), "{log}");
}
