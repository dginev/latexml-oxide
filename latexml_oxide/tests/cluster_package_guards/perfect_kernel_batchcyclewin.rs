//! Red/green guards for batch CYCLEWIN: `cycle_guard.rs`'s `MAX_WINDOW` 10 → 32, so a loop whose period is 11 to 32
//! tokens (the gullet) or boxes (the stomach) ends as the cycle Fatal instead of running to the 180 s deadline or the
//! box-list cap. Repros under `tools/perfect_kernel/repros/expansion-primitives/`. A finite counter loop of ~25 tokens
//! (`\loop … \ifnum\n<2500000 \repeat`) is no cycle, the register values read being in the fingerprint: pinned by
//! `perfect_kernel_batch65f::repeated_file_text_is_no_expansion_loop`, whose fixture warms up with one.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The log of `tex` converted under ar5iv.
fn convert_log(tex: &str) -> String {
  latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty")).0
}

/// CYCLEWIN: fifteen macros in a ring re-expand through a 15-token period with a level pushback (the shape of the
/// apacite/natbib name-call loop before LOOPFIX, 1909.02387, 2212.08776); it ran to `Fatal:Timeout:Convert` at 180 s.
/// Repro expansion-primitives/cycle_guard_long_period. Distinct from batch65f's period-2 loop: a period past the old
/// window, on the gullet's guard.
#[test]
fn a_fifteen_token_expansion_loop_is_a_cycle_fatal() {
  let log = convert_log(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/cycle_guard_long_period.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains(
      "Fatal:Timeout:Recursion Infinite expansion loop: a window of 15 token(s) repeated 100+ times"
    ),
    "{log}"
  );
}

/// CYCLEWIN: `\def\x{abcdefghijk\x}\x` digests an 11-box period from an 11-box input period; the stomach's guard
/// shares the window, so it ran to the box-list cap (`Fatal:Stomach:MemoryBudget`, 2.67M boxes). Repro
/// expansion-primitives/cycle_guard_box_period_eleven. Distinct: the stomach's guard and its span check.
#[test]
fn an_eleven_box_digestion_loop_is_a_cycle_fatal() {
  let log = convert_log(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/cycle_guard_box_period_eleven.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains(
      "Fatal:Stomach:Recursion Infinite digestion loop: a window of 11 box(es) repeated 100+ times from input that \
       repeats every 11 box(es)"
    ),
    "{log}"
  );
}

/// CYCLEWIN: a finite counter `\loop` printing the same nine, then eleven, letters 6,000 times re-reads the same input
/// each period, so its box spans repeated and the stomach's guard called the first loop a cycle (`Fatal:Stomach:
/// Recursion`, a 9-box window); a span now carries the register values read for its box, and the moving `\n` breaks
/// the period. Repro expansion-primitives/cycle_guard_counter_loop_boxes. Distinct: the span salt, the stomach's side
/// of the value fingerprint (the gullet's is pinned by batch65f's `\loop` warm-up).
#[test]
fn a_finite_counter_loop_of_boxes_is_no_cycle() {
  let (log, xml) = latexml::util::test::convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/cycle_guard_counter_loop_boxes.tex"
    ),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  let text = format!(
    "{}{}done",
    "abcdefghi".repeat(6000),
    "abcdefghijk".repeat(6000)
  );
  assert_element(&xml, "p", &[], &format!("<p xml:id=\"p1.1\">{text}</p>"));
}
