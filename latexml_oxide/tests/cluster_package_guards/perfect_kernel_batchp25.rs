//! Guards for P25 (Option C step P2-5): the small `\and` gates and cuts of l_aa, ecai, interact, pasj00, gretsi and
//! lipics find a separator by meaning as well as by spelling (`latexml_contrib/src/and_by_meaning.rs`, the separator
//! query `argument_separators`, OXIDIZED_DESIGN_DIVERGENCES #995): a written `\and` parts as before, and an `\and` held
//! in a macro of the document's, which the spelling missed, parts too. One guard per site, each with the written row
//! (today's output, pinned on int158a before the change) and the macro-held row. Red on int158a: l_aa's gate (both
//! authors got one institute "Univ A and Univ B", 1 Warning for the "and" no block prints), lipics's affiliations (one
//! "Uni X and Inst Y", 1 Warning), interact's `\name` (Ann Able with no affiliation) and `\affil` (two affiliations
//! where the class prints "Univ A and Univ B"). Green on int158a, the reading cutting downstream: ecai's gate, pasj00's
//! cut, gretsi's trim, interact's `\and` before a later `\name`.
//!
//! Each converts its repro under `tools/perfect_kernel/repros/class-bindings/and_by_meaning/` and asserts no Error or
//! Warning and every creator's name and contacts in order.

use super::perfect_kernel_batchp24::assert_rows;

/// A repro of the and_by_meaning directory and its name.
macro_rules! row {
  ($name:literal) => {
    (
      $name,
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/class-bindings/and_by_meaning/",
        $name,
        ".tex"
      )),
    )
  };
}

/// l-aa's `\institute` gate: a macro-held `\and` makes the list numbered institutes, one per author's `\inst`.
#[test]
fn l_aa_institute_gate_reads_a_macro_held_and() {
  assert_rows(&[row!("l_aa_written_and"), row!("l_aa_macro_held_and")], &[
    &["Ann Able", "affiliation=Univ A"],
    &["Bob Baker", "affiliation=Univ B"],
  ]);
}

/// ecai's `\author` gate: a block parted by a macro-held `\and` is a per-author list.
#[test]
fn ecai_author_gate_reads_a_macro_held_and() {
  assert_rows(&[row!("ecai_written_and"), row!("ecai_macro_held_and")], &[
    &["Ann Able"],
    &["Bob Baker"],
  ]);
}

/// interact's `\author`: an `\and` in a `\name` or an `\affil` prints the class's "and " (interact.cls:266), one in
/// front of a later `\name` nothing.
#[test]
fn interact_author_and_is_the_class_word() {
  assert_rows(
    &[
      row!("interact_name_written_and"),
      row!("interact_name_macro_held_and"),
    ],
    &[&["Ann Able", "affiliation=Univ A"], &[
      "Bob Baker",
      "affiliation=Univ A",
    ]],
  );
  assert_rows(
    &[
      row!("interact_affil_written_and"),
      row!("interact_affil_macro_held_and"),
    ],
    &[&["Ann Able", "affiliation=Univ A and Univ B"]],
  );
  assert_rows(
    &[
      row!("interact_top_written_and"),
      row!("interact_top_macro_held_and"),
    ],
    &[&["Ann Able", "affiliation=Univ A"], &[
      "Bob Baker",
      "affiliation=Univ B",
    ]],
  );
}

/// pasj00's author list cut: a macro-held `\and` parts the names as a written one.
#[test]
fn pasj00_author_list_cut_at_a_macro_held_and() {
  assert_rows(
    &[row!("pasj00_written_and"), row!("pasj00_macro_held_and")],
    &[&["Ann Able", "altaffiliation=Univ A"], &[
      "Bob Baker",
      "altaffiliation=Univ B",
    ]],
  );
}

/// gretsi v1.3's `\auteurs`: a macro-held `\and` between the `\auteur` calls is the list's separator, trimmed.
/// (The names hold their affiliation mark, as today.)
#[test]
fn gretsi_auteurs_trim_a_macro_held_and() {
  assert_rows(
    &[row!("gretsi_written_and"), row!("gretsi_macro_held_and")],
    &[&["Ann Able 1", "email=ann@a.edu", "affiliation=Univ A"], &[
      "Bob Baker 2",
      "email=bob@b.edu",
      "affiliation=Univ B",
    ]],
  );
}

/// lipics's affiliations slot: a macro-held `\and` parts it as a written one.
#[test]
fn lipics_affiliations_part_at_a_macro_held_and() {
  assert_rows(
    &[row!("lipics_written_and"), row!("lipics_macro_held_and")],
    &[&["Ann Able", "affiliation=Uni X", "affiliation=Inst Y"], &[
      "Bob Baker",
      "affiliation=Uni Z",
    ]],
  );
}
