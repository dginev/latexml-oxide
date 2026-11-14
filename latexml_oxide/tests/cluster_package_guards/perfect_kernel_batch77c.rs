//! Red/green guard for batch 77c: a shipped class that redefines `\@pushfilename` with the LaTeX2e kernel's pre-2020
//! body is read with the pop of the same era, and the kernel's push and pop return after it (pasa_cls.rs; pasa.cls:45-52,
//! witnesses 1307.3127, 1501.02922, 1510.05346, 1703.06635, 1802.01188, 1407.1937, 1607.08620, 1602.04538, 1401.2223,
//! 1705.02067, 1609.00634, 1511.02959, 1703.03169 — every Error of each was "Extra \PopDefaultHookLabel"). The guard
//! asserts the three measures: no Error/Warning, the authored text present once, in its paragraph.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// pasa.cls's era push paired with latex.ltx's 2020-10 pop popped a hook label per package load (the class's own
/// `\usepackage{graphicx}`, then the document's `\usepackage{amsmath}`); with the pasa binding every load balances.
#[test]
fn pasa_era_pushfilename_balances_package_loads() {
  let (log, xml) = convert_files_with(
    include_str!("../../../tools/perfect_kernel/repros/loader/pasa_era_pushfilename_balanced.tex"),
    &[(
      "pasa.cls",
      include_str!("../../../tools/perfect_kernel/repros/loader/pasa.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("PopDefaultHookLabel"), "{log}");
  assert_eq!(xml.matches("Body text.").count(), 1, "{xml}");
  assert!(xml.contains(r#"<p xml:id="p1.1">Body text.</p>"#), "{xml}");
}
