//! Red/green guards for batch CYCLEPERSIST: the stomach's box-cycle guard Fatals on a period only once it has held
//! (OD #538) — for 50,000 boxes when the input repeats with its register values, for 2,000,000 tokens read when it
//! repeats only register values aside, and then only in a window of at most 10 boxes (0.8.0's window) — so a runaway
//! whose counter moves is caught again (CYCLEWIN's span salt hid it: 0805.0686, 0808.1477, 1812.11491, math0609583),
//! a finite run of repeated data stays clear (2210.07754), and so does a finite 12-box counter loop past the hold.
//! CYCLEWIN's finite counter loop (`perfect_kernel_batchcyclewin::a_finite_counter_loop_of_boxes_is_no_cycle`) reads
//! 190k tokens in its run and stays clear. Repros under `tools/perfect_kernel/repros/expansion-primitives/`.
use std::collections::HashMap;

use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The document a salvage writes for a runaway list past 50,000 boxes: the XML declaration alone, 39 bytes.
const STUB: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";

/// CYCLEPERSIST: `\loop\hbox{a}\hbox{b}\advance\n1 \ifnum\n>0 \repeat` repeats a 2-box window from input that repeats
/// but for the moving `\n`; with the value in every span it was no period, and the run ended at `Fatal:oom` (no
/// `--max-memory` under `ulimit -v`) or the 2.67M box cap. Repro expansion-primitives/cycle_guard_counter_runaway_boxes.
/// Distinct: the moving class and its token clock.
#[test]
fn a_runaway_counter_loop_of_boxes_is_a_cycle_fatal() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/cycle_guard_counter_runaway_boxes.tex"
    ),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    log.contains(
      "Fatal:Stomach:Recursion Infinite digestion loop: a window of 2 box(es) repeated 100+ times from input that \
       repeats every 2 box(es) but for the register values read, for 2"
    ),
    "{log}"
  );
  // Written, as the stub a salvage of a runaway list past 50,000 boxes leaves (`stomach::salvage_pending_box_lists`).
  assert_eq!(xml, STUB);
}

/// CYCLEPERSIST review: `\loop` over twelve `\hbox`es 30,000 times, a 12-box window whose input repeats but for the
/// moving `\n`, holds past 2,000,000 tokens after the activation and ends; 0.8.0's window of 10 never saw it, round 0's
/// moving class Fatalled it with the 39-byte stub. Repro expansion-primitives/cycle_guard_counter_loop_twelve_boxes.
/// Distinct: the moving class's window bound.
#[test]
fn a_finite_twelve_box_counter_loop_is_no_cycle() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/cycle_guard_counter_loop_twelve_boxes.tex"
    ),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(xml.contains("Before.") && xml.contains("After."), "{xml}");
}

/// CYCLEPERSIST review, the accepted bound of OD #538: `\loop\hbox{a}\hbox{b}` 150,000 times under a counter is a
/// 2-box window in sight of the moving class, which calls it a loop after the 2,000,000-token hold, between 80,000 and
/// 85,000 iterations (0.8.0 at the 50,000-box activation, 25,000; pdflatex finishes it). Repro
/// expansion-primitives/cycle_guard_counter_loop_two_boxes_long.
#[test]
#[ignore = "RED: OD #538's accepted bound (a finite counter loop of at most 10 boxes past the moving hold Fatals)"]
fn a_long_finite_two_box_counter_loop_is_no_cycle_red() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/cycle_guard_counter_loop_two_boxes_long.tex"
    ),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains("Before.") && xml.contains("After."), "{xml}");
}

/// CYCLEPERSIST: a pgfplots table whose last 1,000 rows are one row (2210.07754's tail) digests a 13-box window from
/// input that repeats register values and all, for 11k boxes inside a minipage whose list is past 50,000; it read as
/// a loop (`Fatal:Stomach:Recursion`, the plot lost). Repro expansion-primitives/cycle_guard_repeated_plot_rows.
/// Distinct: the stuck class's hold, on finite data.
#[test]
fn a_plot_of_repeated_rows_is_no_cycle() {
  let rows = (0..4000)
    .map(|i| format!("0.{:06} 0.500000\n", 250_000 - 60 * i))
    .chain(std::iter::repeat_n("0.000010 0.999737\n".to_string(), 1000))
    .collect::<String>();
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/cycle_guard_repeated_plot_rows.tex"
    ),
    &[("rows.dat", &rows)],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  // The axis frame and a mark per row: 4,000 distinct marks, and one mark drawn 1,000 times.
  let mut marks: HashMap<&str, usize> = HashMap::new();
  for path in xml.split("<svg:path d=\"").skip(1) {
    let (d, rest) = path.split_once('"').unwrap();
    if rest.starts_with("/>") {
      *marks.entry(d).or_default() += 1;
    }
  }
  assert_eq!(marks.values().sum::<usize>(), 5001, "{xml}");
  assert_eq!(marks.len(), 4002, "{xml}");
  assert_eq!(marks.values().max(), Some(&1000), "{xml}");
  assert!(
    xml.contains("<p xml:id=\"p1.1\">Before.") && xml.contains("</inline-block>\nAfter.</p>"),
    "{xml}"
  );
}
