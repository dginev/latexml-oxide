//! Guards for NL1, the names line of the one engine's reading ported from the old reader's procedure
//! (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`; the rule table RT3, rows N1-N3, C2-C5, S1, A2): each fixture
//! converted with the emitter's switch off and at `LATEXML_FM_EMIT=2`, the creators (name and every contact, in order)
//! the same both ways. One guard per rule: a marked name (N1-N3), glued initials (S1), a collaboration remark (C2-C5),
//! a comma a command draws inside a word (A2).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// `tex` with `files` beside it converted under `emit`: the log and the XML.
fn run_with(tex: &str, files: &[(&str, &str)], emit: u8) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    files,
    Some("ar5iv.sty"),
    move || {
      set_fm_shadow_override(Some(true));
      set_fm_emit_override(Some(emit));
    },
    |_| (),
  );
  (log, xml)
}

/// `tex`'s creators off and at `=2`, each run clean (`warnings` each way).
fn both(tex: &str, warnings: (usize, usize)) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
  both_with(tex, &[], warnings)
}

/// [`both`] with `files` beside `tex`.
fn both_with(
  tex: &str,
  files: &[(&str, &str)],
  warnings: (usize, usize),
) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
  let (off_log, off) = if files.is_empty() {
    run(tex, 0)
  } else {
    run_with(tex, files, 0)
  };
  let (log, xml) = if files.is_empty() {
    run(tex, 2)
  } else {
    run_with(tex, files, 2)
  };
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, warnings.0),
    "{off_log}"
  );
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings.1),
    "{log}"
  );
  (creators(&off), creators(&xml))
}

/// Rows N1-N3: a names line's piece carrying its own mark is a name whatever its words — the generic split never reads
/// an institution word (`R. New$^2$`, 0904.4122; `S. Michelle Consiglio,$^{1}$`, 2103.02641; `Alexander New$^2$`,
/// 2311.16860), and a per-author class's line a surname after an initial (`Gary P. Centers,$^{1}$`, revtex,
/// 1901.10843). Red (MV1 `=2`): each the name before's affiliation. An unmarked piece keeps the reading's institution
/// (`John Smith, University of Toronto`, the main session's decision 2026-11-08), off a creator of its own.
#[test]
fn a_marked_name_with_an_institution_word_is_a_name() {
  let (off, two) = both(
    repro!("emit_a_marked_name_with_an_institution_word_is_a_name"),
    (0, 0),
  );
  let want = [
    vec!["A. M. Broomhall", "affiliation=Univ A"],
    vec!["S. Michelle Consiglio", "affiliation=Univ A"],
    vec!["R. New", "affiliation=Univ B"],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
  let (off, two) = both(
    repro!("revtex_marked_list_surname_after_initial_is_a_name"),
    (0, 0),
  );
  let alpha = "affiliation=University of Alpha";
  let beta = "affiliation=University of Beta";
  assert_eq!(two, [
    vec!["Ann Able", alpha],
    vec!["Bob Baker", alpha],
    vec!["Gary P. Centers", alpha],
    vec!["Dan Dee", beta],
    vec!["Eve Eden 1,2", alpha, beta],
  ]);
  assert_eq!(off.len(), 5);
  let (off, two) = both(
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^1$, University of \
     Toronto\\\\ $^1$Univ A}\n\\maketitle\nx\n\\end{document}\n",
    (0, 0),
  );
  assert_eq!(off, [vec!["Ann Able", "affiliation=Univ A"], vec![
    "University of Toronto"
  ],]);
  assert_eq!(two, [vec![
    "Ann Able",
    "affiliation=University of Toronto",
    "affiliation=Univ A"
  ]]);
}
