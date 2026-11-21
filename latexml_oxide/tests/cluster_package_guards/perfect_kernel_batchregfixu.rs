//! Red/green guards for batch REGFIXU: a stand-in binding's dependency scan of the paper's shipped class runs after
//! the copy's own read (TeX's order), so the copy loads its packages in its order (2211.09062). The repro is checked
//! against pdflatex.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// sn-jnl.cls:297 `\usepackage{easyReview}` before :312 `\usepackage{program}` (2211.09062): the copy's unbound
/// easyReview loads todonotes and tikz with `|` still other, as in pdflatex, and program after it; the sn-jnl binding's
/// scan, run at the class's end, adds nothing the copy loaded. Red (INT6B): the scan loaded the bound program before the
/// copy was read, pgfmathparser.code.tex read with `|` active (30 errors; the witness 295 and a Fatal). The programbox
/// `\(` stays the kernel's inline maths (OXIDIZED_DESIGN_DIVERGENCES #597), now set at the class's end. Distinct: the
/// scan's place in the class load (`\lx@sn@class@end`); batch67u's guard covers what the scan loads, batch68f's what it
/// skips. Repro stand-in-copies/scan_order/snjnl_scan_after_the_copy.
#[test]
fn snjnl_scan_runs_after_the_copy_loads_its_packages() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/stand-in-copies/scan_order/snjnl_scan_after_the_copy.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // easyReview (no binding) before program (bound): the copy's order
  let easy = log.find("easyReview.sty").expect("easyReview read");
  let program = log.find("(Loading program_sty.rs").expect("program loaded");
  assert!(easy < program, "{log}");
  assert!(xml.contains("Check it."), "{xml}");
  assert!(xml.contains(r#"<Math mode="inline" tex="a&lt;b""#), "{xml}");
  assert!(!xml.contains("ltx_minipage"), "{xml}");
}
