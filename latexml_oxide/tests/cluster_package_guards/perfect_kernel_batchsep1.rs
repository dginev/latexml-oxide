//! Guards for SEP1: the rehearsal binds a `\let` copy of an author-line command by its meaning
//! (`observer/rehearsal.rs` `let_copies_of_bound_commands`, OXIDIZED_DESIGN_DIVERGENCES #994). TeX's `\let\nx\and` makes
//! `\nx` the separator; bound by name only, the copy kept the kernel `\and`'s meaning (a constructor printing " and "),
//! the block held no `Sep(And)` and the names merged into one creator, where pdflatex and Perl give two (OC1's `\let`
//! probe). The copies of `\thanks` and `\footnote` take the same path: each is its command's note.
//!
//! Each converts its repro and asserts the exact Error and Warning counts and every creator, in order: the names and
//! contacts for the separators, the whole element for the notes (a `\thanks` note is no contact).

use latexml::util::test::{convert_files_with, error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
};

/// A repro and its name, for a table's failure message to say which row broke.
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

fn owned(expected: &[&[&str]]) -> Vec<Vec<String>> {
  expected
    .iter()
    .map(|c| c.iter().map(|s| s.to_string()).collect())
    .collect()
}

/// A `\let` copy of `\and` parts the names on the kernel's route — article's `\author`, IEEEtran journal's
/// `\lx@add@authors` — spelled in the line or held in a macro of the document's own (`\author{\aut}`, the search reading
/// into the document's macros), or taken before the document renews `\and` (`\let\nx\and\renewcommand\and{…}`: the copy
/// keeps the separator, compared with no current `\and`; SEP1: one merged creator, revSEP1 finding 1). The macro row's
/// names are written from the macro's expansion read in its place (AUT1; two by-text Warnings before, as OC1's
/// macro-held rows).
#[test]
fn a_let_copy_of_and_parts_the_names() {
  for ((name, tex), counts) in [
    (
      row!(
        "sectioning-frontmatter",
        "author_let_copy_of_and_parts_the_names"
      ),
      (0, 0),
    ),
    (
      row!(
        "sectioning-frontmatter",
        "author_let_copy_of_and_parts_the_names_ieeetran"
      ),
      (0, 0),
    ),
    (
      row!(
        "sectioning-frontmatter",
        "author_let_copy_of_and_in_a_macro_parts_the_names"
      ),
      (0, 0),
    ),
    (
      row!(
        "sectioning-frontmatter",
        "author_let_copy_of_and_before_renewal_parts_the_names"
      ),
      (0, 0),
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      counts,
      "{name}: {log}"
    );
    assert_eq!(
      creators(&xml),
      owned(&[&["Ann Able"], &["Bob Baker"]]),
      "{name}: {xml}"
    );
  }
}

/// An eptcs `\author` with a line under each name and a `\let` copy of `\and` between them: the binding's cells see one
/// cell, the block goes whole to the reading (OD #991), which cuts it at the copy's `Sep(And)`, each line its own
/// author's (OC1b: one creator holding "Univ A and Bob Baker").
#[test]
fn eptcs_cells_part_at_a_let_copy_of_and() {
  let class = include_str!("../../../tools/perfect_kernel/repros/class-bindings/eptcs/eptcs.cls");
  let (log, xml) = convert_files_with(
    repro!(
      "class-bindings/eptcs",
      "eptcs_author_let_copy_of_and_parts_the_names"
    ),
    &[("eptcs.cls", class)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    owned(&[&["Ann Able", "affiliation=Univ A"], &[
      "Bob Baker",
      "affiliation=Univ B"
    ]]),
    "{xml}"
  );
}

/// A `\let` copy of `\thanks` (acl.sty's `\let\Thanks\thanks`, 2407.16574) or of `\footnote` in the author line is
/// its command's note on the author, as the command written out is (OC1b: the `\thanks` copy's note lost from the
/// creator; the `\footnote` copy's text read into the name, "Ann Able * * footnote * Univ A"). So is a copy taken before
/// the document renews `\thanks` (`\let\oldthanks\thanks\renewcommand\thanks[1]{}`; SEP1: the note lost from the creator,
/// revSEP1 finding 1).
#[test]
fn a_let_copy_of_a_note_command_is_its_note() {
  for ((name, tex), first) in [
    (
      row!(
        "sectioning-frontmatter",
        "author_let_copy_of_thanks_is_the_note"
      ),
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note class=\"ltx_note_frontmatter \
       ltx_thanks_note\" role=\"thanks\">Univ A, ann@a.org</note>\n  </creator>",
    ),
    (
      row!(
        "sectioning-frontmatter",
        "author_let_copy_of_thanks_before_renewal_is_the_note"
      ),
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note class=\"ltx_note_frontmatter \
       ltx_thanks_note\" role=\"thanks\">Univ A, ann@a.org</note>\n  </creator>",
    ),
    (
      row!(
        "sectioning-frontmatter",
        "author_let_copy_of_footnote_is_the_note"
      ),
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Note:\u{a0}\" \
       role=\"note\">Univ A</contact>\n  </creator>",
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert_eq!(
      creators_xml(&xml),
      vec![
        first.to_string(),
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n  </creator>"
          .to_string()
      ],
      "{name}: {xml}"
    );
  }
}
