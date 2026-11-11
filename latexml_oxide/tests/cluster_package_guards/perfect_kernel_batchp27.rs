//! Guards for P27, Option C step P2-7 (`agent_reports/2026-11-10_P2P_report.md`; OXIDIZED_DESIGN_DIVERGENCES #993's P2
//! paragraph, #995): authblk's two `\author` gates (`authblk_sty.rs` `author_call`, `lines_of_names_before_affil`) ask
//! the separator query whether the names hold a kernel `\and` by meaning (`separated_by_and`, `argument_separators`)
//! instead of testing the token's spelling; and the reading's `annotate=run:<cap>` counts `\author` calls, the cap the
//! binding counts (`resolver/api.rs` `record_run`, `calls_from`), so a list the reading parts into several names before
//! an `\affil` no longer ends the run early. One guard per path: the run cap (a name before a list, two lists — a
//! comma and a kernel `\and`), and the gates' shapes that read the same on both sides (a `\let` copy and a `\csname`, a
//! document's renewed `\and` — trap T8 —, a macro-held `\and` with a line under the names before an `\affil`, the second
//! gate). The macro-held `\and` that now takes the list path is `perfect_kernel_batchoc1c`'s
//! `authblk_two_alike_macro_held_calls_each_own_affil` row (its false content-check excess Warning gone). Each asserts no
//! Error, the exact Warning count, and every creator's name and contacts in order.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

/// A sectioning-frontmatter repro and its name, for a table's failure message to say which row broke.
macro_rules! row {
  ($name:literal) => {
    (
      $name,
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $name,
        ".tex"
      )),
    )
  };
}

/// A row: the repro (name, source), its Warning count, and its creators (name, then contacts in order).
type Row<'a> = ((&'a str, &'a str), usize, &'a [&'a [&'a str]]);

/// Each row converted: no Error, its Warning count, and its creators as expected.
#[track_caller]
fn assert_rows(rows: &[Row]) {
  for ((name, tex), warnings, expected) in rows {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, *warnings),
      "{name}: {log}"
    );
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|c| c.iter().map(|s| s.to_string()).collect())
      .collect();
    assert_eq!(creators(&xml), expected, "{name}: {xml}");
  }
}

/// authblk's unlabelled `\affil` reaches every author of its run, the `\author` calls queued since the last `\affil`
/// (`annotate=run:<calls>`), each call's names whole: a name before a comma list (`Cy Cole`, then `Ann Able, Bob
/// Baker`), and two lists (a comma, a kernel `\and`). Red on int158a: a cap counting the reading's names gave Cy Cole,
/// and Ann Able and Bob Baker, none, silently.
#[test]
fn authblk_affil_run_counts_author_calls() {
  assert_rows(&[
    (row!("authblk_affil_run_name_before_a_list"), 0, &[
      &["Cy Cole", "affiliation=Univ X"],
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Univ X"],
      &["Dee Dunn", "affiliation=Univ Y"],
    ]),
    (row!("authblk_affil_run_two_lists"), 0, &[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Univ X"],
      &["Cy Cole", "affiliation=Univ X"],
      &["Dee Dunn", "affiliation=Univ X"],
    ]),
  ]);
}

/// authblk's gates by meaning, on the shapes whose creators the spelling test already gave: a `\let` copy and a
/// `\csname and\endcsname` (kernel separators: the list path), a document's renewed `\and` (no kernel column: the
/// creator call, which the emitter cuts at its own `\and`), and a macro-held `\and` with a line under the names before
/// an `\affil` (the second gate: a column's line, not rows of names; the macro's names written by their text, two
/// counted Warnings). Green on int158a and P27. The last row pins today's output, not a ruling: `Cy Cole` is Bob
/// Baker's affiliation, where pdflatex prints it as a line in his column (no word lost; revP27 N3).
#[test]
fn authblk_and_gates_read_the_separator_by_meaning() {
  assert_rows(&[
    (row!("authblk_and_let_copy_and_csname_part_names"), 0, &[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Univ X"],
      &["Cy Cole", "affiliation=Univ Y"],
      &["Dee Dunn", "affiliation=Univ Y"],
    ]),
    (row!("authblk_renewed_and_is_no_kernel_column"), 0, &[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Univ X"],
    ]),
    (row!("authblk_macro_held_and_line_before_affil"), 2, &[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Cy Cole", "affiliation=Univ X"],
    ]),
  ]);
}
