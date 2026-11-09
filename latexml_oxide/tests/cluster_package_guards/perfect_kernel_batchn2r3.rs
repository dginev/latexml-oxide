//! Guards for N2r3 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, "The safety net", bar (2); OXIDIZED_DESIGN_DIVERGENCES #980;
//! the main session's rulings on N2r2): the content check on the placed output (`resolver::placed`, under the switch's `=2`
//! only) counts a shared item as often as the inks print its text, at least the most copies one creator holds and at most
//! its copies in all (`a_line_printed_per_author_is_no_loss`; the word lost beside a shared item printed once stays counted,
//! `perfect_kernel_batchn2r2::a_shared_copy_hides_no_differently_worded_loss`); a contact's label (its `name`) is the label's words, never
//! its text's (`a_class_label_in_a_name_is_no_excess`, 2307.11138); and under `=2` the flush reads every paper as the census
//! does, so the check reads the creator calls of a paper with no author block in production too
//! (`the_calls_are_read_without_the_census`, 1401.5951). Each a mechanism of its own: the shared item's count, the label's
//! place, the flush's reading.

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators, perfect_kernel_batchn2::content_checks,
  perfect_kernel_batchz4::run,
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
  let (log, xml, ()) =
    convert_files_with_setup_then(tex, &[], Some("ar5iv.sty"), move || {}, |_| ());
  (log, xml)
}

/// A line printed under each author and kept as one shared item is no loss: `IDSIA Lab, Lugano` printed twice counts twice,
/// its two copies in the XML (N2r2's lost flags, 479 of 547 on dev3; `Email: rezakord@ipm.ir` in two notes, 0712.1873). Each
/// print is taken once, the longest entries first, a line with its separators' words: revtex's two-line `\address` and
/// another author's first line of it are no excess (cond-mat0305063); a print read by its words, an address beside them the
/// e-mail's: IEEEtran's `\IEEEauthorblockA` under each author (2311.03657, 2003.10159).
#[test]
fn a_line_printed_per_author_is_no_loss() {
  let (log, xml) = run(repro!("emit_content_a_line_printed_per_author"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(content_checks(&log), (vec![], vec![]), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "affiliation=IDSIA Lab, Lugano"]),
    row(&["Bob Baker", "affiliation=IDSIA Lab, Lugano"]),
  ]);
  let (log, xml) = run(repro!("emit_content_a_line_inside_a_longer_entry"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(content_checks(&log), (vec![], vec![]), "{log}");
  let two_lines = "affiliation=Univ A, Town Science and Technology Lab, City";
  assert_eq!(creators(&xml), [
    row(&["Ann Able", two_lines]),
    row(&["Bob Baker", "affiliation=Univ A, Town"]),
    row(&["Cat Cole", two_lines]),
  ]);
  let (log, xml) = run(repro!("emit_content_a_line_beside_its_address"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(content_checks(&log), (vec![], vec![]), "{log}");
  assert_eq!(creators(&xml), [
    row(&[
      "Ann Able",
      "affiliation=Univ A Town, Country",
      "email=ann@a.org"
    ]),
    row(&[
      "Bob Baker",
      "affiliation=Univ A Town, Country",
      "email=bob@a.org"
    ]),
  ]);
}

/// A contact's label, its `name`, is the label's words, never its text's: the document's `\email` printing `Email: `, which
/// the rehearsal's `\email` doesn't print, is no excess (2307.11138's mpi2015-cscpreprint.cls, 1602.07570). The label is
/// the contact's `name`, printed once: the content, the `\email` call, drops it (CL10; it read `name="Email: "` beside
/// `Email: ann@a.org`, the label rendered twice — review revX2 2).
#[test]
fn a_class_label_in_a_name_is_no_excess() {
  let (log, xml) = run(repro!("emit_content_a_class_label_in_a_name"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(content_checks(&log), (vec![], vec![]), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "affiliation=Univ A, Town", "email=ann@a.org"]),
    row(&["Bob Baker", "affiliation=Univ B, City", "email=bob@b.org"]),
  ]);
  assert_eq!(
    xml
      .matches("<contact name=\"Email: \" role=\"email\">")
      .count(),
    2,
    "{xml}"
  );
}

/// Under `=2` the flush reads every paper as the census does: llncs's `\institute`, a creator call of a paper with no author
/// block, is rehearsed without the census (deleted in DEL4), so the content check counts the `ann@a.org` it prints twice and the
/// output holds once, the same output and the same count with the census and without it (1401.5951: no finding without
/// it). Switched off, nothing counts.
#[test]
fn the_calls_are_read_without_the_census() {
  let tex = repro!("emit_content_calls_read_without_the_census");
  let expected = [
    row(&["Ann Able", "affiliation=Univ A, Town", "email=ann@a.org"]),
    row(&["Bob Baker", "affiliation=Univ A, Town"]),
  ];
  let (census_log, census_xml) = run(tex);
  let (log, xml) = run_default(tex);
  for log in [&census_log, &log] {
    assert_eq!((error_count(log), warning_count(log)), (0, 1), "{log}");
    assert_eq!(
      content_checks(log),
      (vec!["ann@a.org".to_string()], vec![]),
      "{log}"
    );
  }
  assert_eq!(creators(&xml), expected, "{xml}");
  assert_eq!(xml, census_xml);
}
