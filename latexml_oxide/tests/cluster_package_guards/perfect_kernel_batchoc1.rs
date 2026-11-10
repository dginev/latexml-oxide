//! Guards for OC1, Option C's step P1 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md` "Segmentation is digested"): an
//! author call whose `\and` its spelling hides — held in a macro (`\author{\aut}`, `\newcommand\nx{\and}`), printed by
//! a document wrapper (`\pair{A}{B}` printing `#1 \and #2`), or spelled `\csname and\endcsname` — parts its authors
//! as TeX does: the class's own split by spelling sees one piece, and the reading cuts the rehearsed block at its own
//! `Sep(And)` events. The bindsplit probes om1, ln1, ie1 and ep1 (`agent_reports/2026-11-07_bindsplit_report.md`, one
//! merged creator at int143b) are green at int156a, before OC1; these pin each route a class's `\author` takes to the
//! reading, which P2 (the bindings no longer pre-splitting by spelling) must keep.
//!
//! Each converts its repro and asserts the exact Error and Warning counts and every creator's name and contacts in
//! order. The shapes still red on the base, each a mechanism of its own, are catalog repros with no guard:
//! `author_let_copy_of_and_parts_the_names` (a `\let` copy of `\and` prints no `Sep(And)` in the rehearsal) and
//! `ieeetran_author_grid_in_a_macro` / `ieeetran_author_grid_through_a_wrapper` (the IEEEtran grid's token split).

use latexml::util::test::{convert_files_with, error_count, warning_count};

use super::{perfect_kernel_batchz4::run, perfect_kernel_batchz18o::assert_creators};

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
/// `\lx@splitting{\lx@add@author@split}{\and\And}`), llncs's whole block and its marked split (`llncs_cls.rs`
/// `\author`, `\lx@llncs@author@split`), the kernel's (`\lx@add@authors`, IEEEtran journal) — is cut at the `\and`
/// the rehearsal digests from a macro, a wrapper or a `\csname`, each `\inst` picking its numbered `\institute`. The
/// three plain-text Warnings of the macro-held rows are the emitter writing a name its events don't spell (a macro's
/// expansion: P3), and OmniBus's missing class binding.
#[test]
fn a_block_parts_at_the_and_its_spelling_hides() {
  for (tex, counts, creators) in [
    (
      repro!(
        "sectioning-frontmatter",
        "author_call_cut_at_a_macro_held_and_omnibus"
      ),
      (0, 3),
      &[&["Ann Able"][..], &["Bob Baker"][..]][..],
    ),
    (
      repro!(
        "sectioning-frontmatter",
        "author_call_cut_at_a_wrapper_held_and_omnibus"
      ),
      (0, 1),
      &[&["Ann Able"][..], &["Bob Baker"][..]][..],
    ),
    (
      repro!(
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
      repro!(
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
      repro!(
        "sectioning-frontmatter",
        "author_call_cut_at_a_csname_and_ieeetran"
      ),
      (0, 0),
      &[&["Ann Able"][..], &["Bob Baker"][..]][..],
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), counts, "{log}");
    assert_creators(&xml, creators);
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
