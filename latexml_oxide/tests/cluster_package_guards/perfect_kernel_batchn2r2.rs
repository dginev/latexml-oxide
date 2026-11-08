//! Guards for N2r2 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, "The safety net", bar (2); OXIDIZED_DESIGN_DIVERGENCES #980;
//! review revN2): the content check on the placed output (`resolver::placed`, under the switch's `=2` only) counts a shared
//! item once per the most copies one creator holds on the lost side too, so its copies never hide a word lost elsewhere
//! (`a_shared_copy_never_hides_a_loss`); a block written whole exempts only the creator it became from the excess count
//! (`a_whole_block_exempts_only_its_own_creator`); every word counts, a mark excluded by its event, not by its length
//! (`a_lost_letter_is_counted`); and the rehearsal's command names never reach the output: a verbatim e-mail argument's
//! `\and` parts its addresses (`an_email_parted_by_and_keeps_its_addresses`, 2106.06418). Each a mechanism of its own: the
//! lost side's copies, the whole block's exemption, the word's definition, the rehearsal's reversion.

use latexml::util::test::{error_count, warning_count};

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

/// A word lost beside a shared item holding it is counted: `IDSIA Lab, Lugano`, printed once and given to both authors of
/// its row, counts once, so the `IDSIA` the second row's line loses is missing (review revN2 finding 1, its `m1.tex`: the
/// shared affiliation's two copies made up for the lost one, no Warning).
#[test]
fn a_shared_copy_never_hides_a_loss() {
  let tex = repro!("emit_content_a_shared_copy_hides_no_loss");
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_eq!(
    content_checks(&log),
    (vec!["IDSIA".to_string()], vec![]),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "affiliation=IDSIA Lab, Lugano"]),
    row(&["Bob Baker", "affiliation=IDSIA Lab, Lugano"]),
    row(&["Cat Cole", "email=cat@idsia.ch"]),
    row(&["Dan Dorn", "email=dan@idsia.ch"]),
    row(&[]),
  ]);
  let (log, _) = run(tex, 1);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(content_checks(&log), (vec![], vec![]), "{log}");
}

/// A block written whole exempts the creator it became, never the rest of the paper: the second author's name, placed in
/// its personname and again as its ORCID link's text, is counted in excess beside the block written whole (review revN2
/// finding 2; the link's text 2203.03523, 2306.13366).
#[test]
fn a_whole_block_exempts_only_its_own_creator() {
  let (log, xml) = run(repro!("emit_content_a_whole_block_exempts_only_itself"), 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert!(
    log.contains("the block is written whole, as Perl's whole-block creator"),
    "{log}"
  );
  assert_eq!(
    content_checks(&log),
    (vec![], vec!["Cat Cole".to_string()]),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    row(&["Ann Able ann@a.org Bob Baker bob@b.org Univ A Town, Country"]),
    row(&["Cat Cole", "orcid=Cat Cole", "affiliation=Univ C"]),
  ]);
}

/// A lost one-letter word is counted: every word counts, a mark excluded by its Mark event, not by its length (review revN2
/// finding 5: a lost initial, a one-letter word or a two-digit number was never counted).
#[test]
fn a_lost_letter_is_counted() {
  let (log, xml) = run(repro!("emit_content_a_lost_letter_is_counted"), 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_eq!(
    content_checks(&log),
    (vec!["B".to_string()], vec![]),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "email=ann@b.ch"]),
    row(&[
      "Bob Baker",
      "affiliation=IDSIA Lab, Lugano",
      "email=bob@b.ch"
    ]),
    row(&[]),
  ]);
}

/// svjour3's `\email{ann@a.org \and bob@b.org}` in an `\institute` is two addresses with either switch, the rehearsal's
/// `\and` reverting as `\and` and parting them as a break does: never a contact `\lx@fm@obs@andbob@b.org` (2106.06418,
/// review revN2 finding 3).
#[test]
fn an_email_parted_by_and_keeps_its_addresses() {
  let tex = repro!("emit_content_an_email_parted_by_and");
  let expected = [
    row(&["Ann Able", "email=ann@a.org"]),
    row(&["Bob Baker", "email=bob@b.org"]),
    row(&["affiliation=Univ A, Town"]),
  ];
  for emit in [0, 2] {
    let (log, xml) = run(tex, emit);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert!(!xml.contains("lx@fm@obs"), "{xml}");
    assert_eq!(creators(&xml), expected, "emit={emit}: {xml}");
  }
}
