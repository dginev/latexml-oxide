//! Guards for OC1, Option C's step P1 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md` "Segmentation is digested"): an
//! author call whose `\and` its spelling hides — held in a macro (`\author{\aut}`, `\newcommand\nx{\and}`), printed by
//! a document wrapper (`\pair{A}{B}` printing `#1 \and #2`), or spelled `\csname and\endcsname` — parts its authors
//! as TeX does: the class's own split by spelling sees one piece, and the reading cuts the rehearsed block at its own
//! `Sep(And)` events. The bindsplit probes om1, ln1, ie1 and ep1 (`agent_reports/2026-11-07_bindsplit_report.md`, one
//! merged creator at int143b) are green at int156a, before OC1; these pin each route a class's `\author` takes to the
//! reading, which P2 (the bindings no longer pre-splitting by spelling) must keep.
//!
//! Each converts its repro and asserts the exact Error and Warning counts and every creator's name and contacts in
//! order. The shapes still red on the base, each a mechanism of its own: `author_let_copy_of_and_parts_the_names` (a
//! `\let` copy of `\and` printed no `Sep(And)` in the rehearsal; green since SEP1, the copy bound by its meaning,
//! `perfect_kernel_batchsep1`) and `ieeetran_author_grid_in_a_macro` / `ieeetran_author_grid_through_a_wrapper` (the
//! IEEEtran grid's token split; green since P28, the routes reading a document macro by meaning,
//! `perfect_kernel_batchp28`).
//! These are the block routes only: an API creator call holding the `\and` (acmart's `\author` with no comma) is the
//! API route's, guarded by `perfect_kernel_batchoc1c` (OD #993 "The API route").

use latexml::util::test::{convert_files_with, error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run,
  perfect_kernel_batchz18o::assert_creators,
};

/// A repro and its name, for a table's failure message to say which row broke (review revOC1 N2).
macro_rules! row {
  ($dir:literal, $name:literal) => {
    ($name, repro!($dir, $name))
  };
}

macro_rules! repro {
  ($dir:literal, $name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $name,
      ".tex"
    ))
  };
}

/// The block a class's `\author` queues, whatever split of its own ran first — OmniBus's (`inst_support_sty.rs`
/// `\lx@add@author@split`, its whole list since P2-1), llncs's whole block and its marked split (`llncs_cls.rs`
/// `\author`, `\lx@llncs@author@split`), the kernel's (`\lx@add@authors`, IEEEtran journal) — is cut at the `\and`
/// the rehearsal digests from a macro, a wrapper or a `\csname`, each `\inst` picking its numbered `\institute`. The
/// three plain-text Warnings of the macro-held rows are the emitter writing a name its events don't spell (a macro's
/// expansion: P3), and OmniBus's missing class binding.
#[test]
fn a_block_parts_at_the_and_its_spelling_hides() {
  for ((name, tex), counts, expected) in [
    (
      row!(
        "sectioning-frontmatter",
        "author_call_cut_at_a_macro_held_and_omnibus"
      ),
      (0, 3),
      &[&["Ann Able"][..], &["Bob Baker"][..]][..],
    ),
    (
      row!(
        "sectioning-frontmatter",
        "author_call_cut_at_a_wrapper_held_and_omnibus"
      ),
      (0, 1),
      &[&["Ann Able"][..], &["Bob Baker"][..]][..],
    ),
    (
      row!(
        "sectioning-frontmatter",
        "author_call_cut_at_a_macro_held_and_llncs"
      ),
      (0, 2),
      &[
        &["Ann Able", "affiliation=Univ A"][..],
        &["Bob Baker", "affiliation=Univ A"][..],
      ][..],
    ),
    (
      row!(
        "sectioning-frontmatter",
        "author_call_cut_at_a_macro_held_and_llncs_marked"
      ),
      (0, 0),
      &[
        &["Ann Able", "affiliation=Univ A"][..],
        &["Bob Baker", "affiliation=Univ B"][..],
      ][..],
    ),
    (
      row!(
        "sectioning-frontmatter",
        "author_call_cut_at_a_csname_and_ieeetran"
      ),
      (0, 0),
      &[&["Ann Able"][..], &["Bob Baker"][..]][..],
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      counts,
      "{name}: {log}"
    );
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|c| c.iter().map(|s| s.to_string()).collect())
      .collect();
    assert_eq!(creators(&xml), expected, "{name}: {xml}");
  }
}

/// The IEEEtran conference grid whose column separator is a macro (`\nx` printing `\and` between two
/// `\IEEEauthorblockN`/`\IEEEauthorblockA` pairs): each block's name and affiliation its own author's (bindsplit
/// ie1: Ann's contacts ended in "and" at int143b).
#[test]
fn ieeetran_grid_parts_at_a_macro_held_and() {
  let (log, xml) = run(repro!(
    "sectioning-frontmatter",
    "author_call_cut_at_a_macro_held_and_ieeetran_grid"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&["Ann Able", "affiliation=Univ A"], &[
    "Bob Baker",
    "affiliation=Univ B",
  ]]);
}

/// An eptcs `\author` with an `\institute` under each name and a macro for the `\and` between them: the binding's
/// cells see one, the block goes whole to the reading (OD #991), and each institute is its own author's (bindsplit ep1).
#[test]
fn eptcs_cells_part_at_a_macro_held_and() {
  let class = include_str!("../../../tools/perfect_kernel/repros/class-bindings/eptcs/eptcs.cls");
  let (log, xml) = convert_files_with(
    repro!(
      "class-bindings/eptcs",
      "eptcs_author_cut_at_a_macro_held_and"
    ),
    &[("eptcs.cls", class)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&["Ann Able", "affiliation=Univ A"], &[
    "Bob Baker",
    "affiliation=Univ B",
  ]]);
}
