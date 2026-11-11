//! Guards for batch P21 (Option C step P2-1, OD #993): OmniBus's `\author` (inst_support_sty.rs), ptephy's
//! (`\lx@ptephy@kernel@author`) and webofc's hand their whole name list to the reading in one `\lx@add@author@split`
//! call, which cuts it at its `\and`s (`Sep(And)`), where each split it by spelling first (`\lx@splitting`); webofc's at
//! its `\\` lines too, every one names by the binding's `break_parts_names` declaration (webofc.cls:497-552). Each
//! repro compiles clean with pdflatex (TL 2025; ptephy with 1403.0174's ptephy.cls, webofc with the cut-down
//! class-bindings/webofc/webofc.cls, OmniBus's `nosuchclass` as `article`); each guard asserts every creator's name and
//! contacts, in order, and the exact Error and Warning counts.

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/author_one_block/",
      $name,
      ".tex"
    ))
  };
}

/// The cut-down webofc.cls the webofc repros ship beside them (the class is not in TeX Live).
const WEBOFC: &str =
  include_str!("../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc.cls");

/// Converts `tex` with the ar5iv preload (beside webofc.cls when `webofc`), asserting its Error and Warning counts:
/// its creators, each its name then its contacts as `role=text`.
fn creators_of(tex: &str, webofc: bool, counts: (usize, usize)) -> Vec<Vec<String>> {
  let files: &[(&str, &str)] = if webofc {
    &[("webofc.cls", WEBOFC)]
  } else {
    &[]
  };
  let (log, xml) = latexml::util::test::convert_files_with(tex, files, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), counts, "{log}");
  creators(&xml)
}

fn rows(rows: &[&[&str]]) -> Vec<Vec<String>> {
  rows
    .iter()
    .map(|row| row.iter().map(ToString::to_string).collect())
    .collect()
}

/// One block per `\author` call, the reading's cut parting its names, on each of the three routes: the list as
/// written (its `\and`, each piece's commas: the same XML as the split by spelling gave) and an `\and` held in a macro, which no spelling shows (the reading cut it inside a piece
/// before P2-1 as well, OC1). The Warnings: OmniBus's missing class binding; the emitter writing the names a macro's
/// expansion spells as plain text (P3), two.
#[test]
fn an_author_list_is_one_block_the_reading_cuts() {
  let literal = rows(&[&["Ann Able"], &["Dan Dee"], &["Bob Baker"]]);
  let hidden = rows(&[&["Ann Able"], &["Bob Baker"], &["Cy Cole"]]);
  let literal_inst = rows(&[
    &["Ann Able", "affiliation=Univ A"],
    &["Dan Dee", "affiliation=Univ B"],
    &["Bob Baker", "affiliation=Univ B"],
  ]);
  let hidden_inst = rows(&[
    &["Ann Able", "affiliation=Univ A"],
    &["Bob Baker", "affiliation=Univ B"],
    &["Cy Cole", "affiliation=Univ A"],
  ]);
  for (name, tex, webofc, counts, expected) in [
    (
      "omnibus literal",
      repro!("omnibus_author_list_one_block_literal_and"),
      false,
      (0, 1),
      &literal,
    ),
    (
      "omnibus hidden",
      repro!("omnibus_author_list_one_block_hidden_and"),
      false,
      (0, 3),
      &hidden,
    ),
    (
      "ptephy literal",
      repro!("ptephy_author_list_one_block_literal_and"),
      false,
      (0, 0),
      &literal,
    ),
    (
      "ptephy hidden",
      repro!("ptephy_author_list_one_block_hidden_and"),
      false,
      (0, 2),
      &hidden,
    ),
    (
      "webofc literal",
      repro!("webofc_author_list_one_block_literal_and"),
      true,
      (0, 0),
      &literal_inst,
    ),
    (
      "webofc hidden",
      repro!("webofc_author_list_one_block_hidden_and"),
      true,
      (0, 2),
      &hidden_inst,
    ),
  ] {
    assert_eq!(&creators_of(tex, webofc, counts), expected, "{name}");
  }
}

/// webofc's `\author` paragraph sets each `\\` line as names (webofc.cls:497-552): a `\\` held in a macro, which the
/// split at `\\` by spelling never saw, parts "The X Collaboration" off as a creator of its own, as a literal one does
/// (78e's `webofc_author_line_break_options`) — the binding's `break_parts_names`. Red (int157a): a collaboration
/// contact of Bob Baker, the line read under the kernel's conventions.
#[test]
fn webofc_a_macro_held_line_break_parts_names() {
  assert_eq!(
    creators_of(
      repro!("webofc_author_list_macro_held_break_parts_names"),
      true,
      (0, 0)
    ),
    rows(&[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ B"],
      &["The X Collaboration"],
    ])
  );
}
