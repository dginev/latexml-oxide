//! Guards for DEL3: the frontmatter emitter writes a block calling a binding's list setter (`\institute` inside the author
//! block) itself — its names from the reading, the setter's call digested as it is after them, its entries the list the
//! names' marks request (`writer::emit::list_setter_runs`) — where it stayed out and the S2b writer wrote it
//! ("a binding's list setter called inside the block"). One guard per binding's setter, each a different definition:
//! sv_support's (`\lx@sv@institute`, its `\and` pieces numbered for `\inst`) in a piece the split `\author` made, and its
//! marked list (each line labelled by its mark, which a hand-typed mark on a name requests); llncs's dispatch
//! (`\lx@llncs@institute@dispatch`) left in a name's group, which `take_institutes` does not take; aa's
//! (`\lx@aa@institute@dispatch`) in the kernel `\author`'s one block; inst_support's own (svproc) in a split piece.
//! Each asserts every creator's name and contacts, in order, and a clean conversion: no Warning (the emitter's stay-out
//! one was the base's).

use super::{
  perfect_kernel_batch46::{convert, error_count, warning_count},
  perfect_kernel_batch84b::creators,
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

/// Converts `tex` (the arXiv configuration) and checks it converts clean with exactly the creators `expected`.
#[track_caller]
fn creators_are(tex: &str, expected: &[&[&str]]) {
  let (log, xml) = convert(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let expected: Vec<Vec<String>> = expected
    .iter()
    .map(|creator| creator.iter().map(ToString::to_string).collect())
    .collect();
  assert_eq!(creators(&xml), expected, "{xml}");
}

/// cond-mat0605195's shape: sv_support's `\institute{Univ X, Town \and Lab Y, City}` in the last piece's group, its pieces
/// numbered for the `\inst` marks; no mark printed in a name.
#[test]
fn batchdel3_sv_support_institute_in_a_piece_numbers_its_list() {
  creators_are(repro!("list_setter_in_block_sv_inst"), &[
    &["Ann Able", "affiliation=Univ X, Town"],
    &["Bob Baker", "affiliation=Lab Y, City"],
    &["Cy Coe", "affiliation=Univ X, Town"],
  ]);
}

/// 2502.07804's shape: the institute's lines each labelled by its mark (`\lx@sv@institute@unlabelled`'s marked list),
/// which the names' hand-typed `${}^{1}$` request, the names parted at the comma by the reading.
#[test]
fn batchdel3_sv_support_marked_institute_lines_go_by_the_names_marks() {
  creators_are(repro!("list_setter_in_block_sv_marked"), &[
    &["Ann Able", "affiliation=Univ X, Town"],
    &["Bob Baker", "affiliation=Lab Y, City"],
  ]);
}

/// llncs's `\institute` in a name's group, which `take_institutes` leaves in the block: its dispatch digested as written.
#[test]
fn batchdel3_llncs_institute_in_a_names_group_numbers_its_list() {
  creators_are(repro!("list_setter_in_block_llncs"), &[
    &["Ann Able", "affiliation=Univ X, Town"],
    &["Bob Baker", "affiliation=Lab Y, City"],
  ]);
}

/// aa's `\institute` in the kernel `\author`'s one block.
#[test]
fn batchdel3_aa_institute_in_the_block_numbers_its_list() {
  creators_are(repro!("list_setter_in_block_aa"), &[
    &["Ann Able", "affiliation=Univ X, Town"],
    &["Bob Baker", "affiliation=Lab Y, City"],
  ]);
}

/// inst_support's own `\institute` (svproc, OmniBus's) in a split piece's group.
#[test]
fn batchdel3_inst_support_institute_in_a_piece_numbers_its_list() {
  creators_are(repro!("list_setter_in_block_inst_support"), &[
    &["Ann Able", "affiliation=Univ X, Town"],
    &["Bob Baker", "affiliation=Lab Y, City"],
  ]);
}
