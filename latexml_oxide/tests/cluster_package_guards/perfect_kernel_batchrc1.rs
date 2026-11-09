//! Guards for RC1 (`agent_reports/2026-11-08_RC1_report.md`; OXIDIZED_DESIGN_DIVERGENCES #980): the charter's content
//! invariant under `LATEXML_FM_EMIT=2` — every word TeX printed in an author block is in the output once, or counted in a
//! Warning. Two mechanisms, a guard each: the gullet's input mark is exact (`gullet::input_unread_since` reads how short
//! the mouth's pushback has been since the mark, its watch's runs, so IEEEtran's affiliation calls are no longer read as
//! the author block's own and lost: 2401.11731, 1801.03314; its unit tests
//! `gullet::token_limit_tests::{an_input_mark_tells_a_refilled_slot_from_a_peek,
//! input_marks_nest_and_keep_no_copy_of_the_pushback}`, and a mark keeps no copy of the pushback:
//! [`a_macro_held_author_list_costs_no_copy_per_mark`]),
//! and the content check counts what TeX printed of every creator call, its words required unless its block's events
//! print them (`resolver::placed::content_check`: a call no rehearsal read counted by its content's text).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::set_fm_shadow_override;

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

/// Convert `tex` without the census (production's default): the log and the XML.
fn run_default(tex: &str) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    &[],
    Some("ar5iv.sty"),
    move || {
      set_fm_shadow_override(None);
    },
    |_| (),
  );
  (log, xml)
}

/// IEEEtran's `\IEEEauthorblockN{…}⏎\IEEEauthorblockA{…}`: the block's mark held the line-end space, read with
/// `\IEEEauthorblockA` and its argument; the affiliation lines' expansion, longer, put another space at the mark's last
/// slot, which a one-token test read as unread, so the affiliation calls were recorded as the block's own, never read, and
/// dropped under `=2` with no Warning (2401.11731, 1801.03314). The mark knows the slot was read (the pushback was
/// shorter than it since): `=2` writes what off writes, `Univ Alpha` Ann Able's.
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
  let (log, xml) = run_default(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), expected);
}

/// A call the rehearsal gave up on (`SkipReason::OverBudget`, a guard's small budget) is read from no rehearsal, and the
/// emitter writes no item for it: its words, its content's text, are counted by the content check, in one Warning beside
/// the budget's, which says what `=2` does with it. Distinct from the mark's guard: the check's side, a call with no events.
#[test]
fn an_unread_calls_words_are_counted() {
  let (log, xml) = in_a_directory(repro!("emit_content_check_counts_an_unread_call"), &[], &[
    ("LATEXML_REHEARSAL_READ_BUDGET", "20000"),
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

/// The process's peak resident set and its resident set now, in kB (`/proc/self/status`'s `VmHWM`, `VmRSS`).
fn resident_kb() -> (u64, u64) {
  let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
  let field = |name: &str| {
    status
      .lines()
      .find_map(|line| line.strip_prefix(name))
      .and_then(|rest| rest.trim().trim_end_matches("kB").trim().parse().ok())
      .unwrap_or(0)
  };
  (field("VmHWM:"), field("VmRSS:"))
}

/// The input marks keep no copy of the pushback: 3,000 `\author`/`\affiliation` pairs a macro holds (revtex4-2's
/// `\authorlist`, revRC1's probe), each block's mark taken while the rest of the list waits on the pushback, cost what
/// a few authors do. A copy per mark made the conversion's peak 3.7 GB against 261 MB (quadratic: 1.1 GB at 1,500);
/// the peak's growth over the conversion stays under 1 GB, every author a creator. Distinct from the gullet's unit
/// test (`gullet::token_limit_tests::input_marks_nest_and_keep_no_copy_of_the_pushback`): the ledger's and the
/// document calls' marks, held for the whole conversion.
#[test]
fn a_macro_held_author_list_costs_no_copy_per_mark() {
  let mut tex = String::from("\\documentclass[aps,prl]{revtex4-2}\n\\def\\authorlist{%\n");
  for n in 0..3000 {
    tex.push_str(&format!(
      "\\author{{Author Name{n} Surname}}\\affiliation{{Department {n}, University Somewhere}}\n"
    ));
  }
  tex.push_str("}\n\\begin{document}\n\\title{T}\n\\authorlist\n\\begin{abstract}A.\\end{abstract}\n\\maketitle\nText.\n\\end{document}\n");
  // (the peak from here on: `/proc/self/clear_refs`' 5 resets it to the resident set now)
  let _ = std::fs::write("/proc/self/clear_refs", "5");
  let (_, before) = resident_kb();
  let (peak, _) = resident_kb();
  eprintln!("peak growth {} kB", peak.saturating_sub(before));
  assert!(
    peak.saturating_sub(before) < 1_000_000,
    "peak {peak} kB, {before} kB before the conversion"
  );
}

/// What a call no rehearsal read printed is counted by words as its ink parts them (`ledger::content_text`): an accent
/// (a control symbol) or a group inside a word parts nothing (`Universit\"at`, `M{\"u}nchen`, `Montr\'{e}al`), nor does a
/// dotless i inside its accent's group or after it (`F\'{\i}sica`, `Bras\'\i lia`) — 6 words, not 12 pieces (`at M
/// unchen Universit`: review revRC1 finding 3; `Bras F lia sica`: review revRC1r2 finding 1); the accents and `\i` unread, the words are spelled without
/// them, and the check takes such a word for the accented one a placed copy prints (`resolver::placed`'s reconcile).
/// Distinct from [`an_unread_calls_words_are_counted`]: the text's parting into words, which a stand-in copy's unrecorded
/// call (`Ledger::unrecorded`) shares.
#[test]
fn an_unread_calls_accented_words_count_whole() {
  let (log, xml) = in_a_directory(
    repro!("emit_content_check_counts_accented_words_whole"),
    &[],
    &[("LATEXML_REHEARSAL_READ_BUDGET", "20000")],
  );
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
  assert_eq!(warning_count(&log), 2, "{log}");
  assert_eq!(
    content_checks(&log),
    (
      vec!["1 Braslia Fsica Montreal Munchen Universitat".to_string()],
      vec![]
    ),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    row(&["Ann Able *"]),
    row(&["Bob Baker †", "affiliation=Univ Beta"]),
  ]);
}

/// The content check's own rehearsals (a call its block's own expansion made, here revtex's `\collaboration` after an
/// `\author`, 2009.10691's shape) read from the check's allowance (`rehearsal::for_the_check`), not the conversion's
/// budget: under a budget the two collaborations' check rehearsals would spend, Cat Cole's `\affiliation` keeps its
/// rehearsal and `=2` writes what the base wrote, with no Warning (review revRC1 finding 4). Distinct: the budget's
/// accounting, no reading or counting of words.
#[test]
fn the_checks_rehearsals_leave_the_budget() {
  let (log, xml) = in_a_directory(repro!("emit_check_rehearsals_leave_the_budget"), &[], &[(
    "LATEXML_REHEARSAL_READ_BUDGET",
    "6000",
  )]);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&[
      "Ann Able",
      "affiliation=Univ Alpha",
      "collaboration=On behalf of the Big Collaboration"
    ]),
    row(&[
      "Bob Baker",
      "affiliation=Univ Beta",
      "collaboration=On behalf of the Bigger Collaboration"
    ]),
    row(&["Cat Cole", "affiliation=Univ Gamma"]),
  ]);
}
