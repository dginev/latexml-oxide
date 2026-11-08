//! Guards for RC1 (`agent_reports/2026-11-08_RC1_report.md`; OXIDIZED_DESIGN_DIVERGENCES #980): the charter's content
//! invariant under `LATEXML_FM_EMIT=2` — every word TeX printed in an author block is in the output once, or counted in a
//! Warning. Two mechanisms, a guard each: the gullet's input mark is exact (`gullet::input_unread_since` compares the
//! pushback below the mark whole, so IEEEtran's affiliation calls are no longer read as the author block's own and lost:
//! 2401.11731, 1801.03314; its unit test `gullet::token_limit_tests::an_input_mark_tells_a_refilled_slot_from_a_peek`),
//! and the content check counts what TeX printed of every creator call, its words required unless its block's events
//! print them (`resolver::placed::content_check`: a call no rehearsal read counted by its content's text).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

use super::{
  perfect_kernel_batch84b::creators, perfect_kernel_batchn2::content_checks,
  perfect_kernel_batchz3::in_a_directory,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// Convert `tex` under `emit` without the census (production's default): the log and the XML.
fn run_default(tex: &str, emit: u8) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    &[],
    Some("ar5iv.sty"),
    move || {
      set_fm_shadow_override(None);
      set_fm_emit_override(Some(emit));
    },
    |_| (),
  );
  (log, xml)
}

/// IEEEtran's `\IEEEauthorblockN{…}⏎\IEEEauthorblockA{…}`: the block's mark held the line-end space, read with
/// `\IEEEauthorblockA` and its argument; the affiliation lines' expansion, longer, put another space at the mark's last
/// slot, which a one-token test read as unread, so the affiliation calls were recorded as the block's own, never read, and
/// dropped under `=2` with no Warning (2401.11731, 1801.03314). The mark compares the pushback below it whole: `=2` writes
/// what off writes, `Univ Alpha` Ann Able's.
#[test]
fn an_affiliation_after_a_refilled_mark_is_read() {
  let tex = repro!("emit_ieee_affiliation_after_a_refilled_mark");
  let expected = [
    row(&["Ann Able *", "affiliation=Univ Alpha"]),
    row(&[
      "Bob Baker †",
      "affiliation=a b c d e f g h i j k l m n o p q r s t u v w x y z",
    ]),
    row(&["affiliation=Univ Line3"]),
  ];
  for emit in [0, 2] {
    let (log, xml) = run_default(tex, emit);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creators(&xml), expected, "emit={emit}");
  }
}

/// A call the rehearsal gave up on (`SkipReason::OverBudget`, a guard's small budget) is read from no rehearsal, and the
/// emitter writes no item for it: its words, its content's text, are counted by the content check, in one Warning beside
/// the budget's, which says what `=2` does with it. Distinct from the mark's guard: the check's side, a call with no events.
#[test]
fn an_unread_calls_words_are_counted() {
  let (log, xml) = in_a_directory(repro!("emit_content_check_counts_an_unread_call"), &[], &[
    ("LATEXML_REHEARSAL_READ_BUDGET", "20000"),
    ("LATEXML_FM_EMIT", "2"),
  ]);
  assert_eq!(
    (error_count(&log), log.matches("\nFatal:").count()),
    (0, 0),
    "{log}"
  );
  assert_eq!(
    log.matches("Warning:frontmatter:rehearsal_budget").count(),
    1,
    "{log}"
  );
  assert!(
    log.contains(
      "any other call's words are not written, the frontmatter content check counting them"
    ),
    "{log}"
  );
  assert_eq!(warning_count(&log), 2, "{log}");
  assert_eq!(
    content_checks(&log),
    (vec!["Alpha Univ".to_string()], vec![]),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    row(&["Ann Able *"]),
    row(&["Bob Baker †", "affiliation=Univ Beta"]),
  ]);
}
