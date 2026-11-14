//! Guards for P24 (Option C step P2-4): eptcs's author block cut where its separators are by meaning
//! (`latexml_contrib/src/eptcs_cls.rs` `cells`, the separator query `argument_separators`, OXIDIZED_DESIGN_DIVERGENCES
//! #991, #995), not by spelling. eptcs.cls sets `\@author` in a centred `tabular`, so its `\and` closes a column and
//! opens one beside it: a macro holding the `\and` or a `\let` copy of it opens a column as the written `\and` does. The
//! binding's cells decide where a column's names start and whether a cell opens with a table of names; its split by
//! spelling saw one cell at a macro-held `\and`, so a table after it was a line under the name before it and a name
//! after a table was glued to the table's last name. One guard per path — a cell after a table, a cell opening with a
//! table, the gate that tells the class's block from the kernel's — each with a macro-held row (red on int157a) and a
//! `\let` row (green on int157a: the split matched an alias of `\and`'s own definition); and a macro holding names and
//! the `\and` together, which no cut by tokens may part (green on int157a). The plain cells' macro-held and
//! `\let` rows are `perfect_kernel_batchoc1::eptcs_cells_part_at_a_macro_held_and` and
//! `perfect_kernel_batchsep1::eptcs_cells_part_at_a_let_copy_of_and`. A keep-guard pins the whole creator elements of
//! today's shapes (the repros of `perfect_kernel_batch84b`, oc1 and sep1), written on int157a before the change.
//!
//! Each converts its repro under `tools/perfect_kernel/repros/class-bindings/eptcs/` beside the repros' stand-in
//! `eptcs.cls` and asserts no Error or Warning and every creator's name and contacts in order.

use latexml::util::test::{convert_files_with, error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::creators_xml};

/// A repro of the eptcs directory and its name, for a table's failure message to say which row broke.
macro_rules! row {
  ($name:literal) => {
    (
      $name,
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/class-bindings/eptcs/",
        $name,
        ".tex"
      )),
    )
  };
}
pub(super) use row;

/// The repros' stand-in class, written from scratch (no code of eptcs.cls).
const CLASS: &str =
  include_str!("../../../tools/perfect_kernel/repros/class-bindings/eptcs/eptcs.cls");

/// `tex` converted beside the stand-in class, with no Error and `warnings` Warnings: its XML.
#[track_caller]
fn convert_warned(name: &str, tex: &str, warnings: usize) -> String {
  let (log, xml) = convert_files_with(tex, &[("eptcs.cls", CLASS)], Some("ar5iv.sty"));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{name}: {log}"
  );
  xml
}

/// `tex` converted beside the stand-in class, checked clean: its XML.
#[track_caller]
fn convert(name: &str, tex: &str) -> String { convert_warned(name, tex, 0) }

/// Each row converts with no Error and `warnings` Warnings to the creators `expected` (name, then each contact as
/// `role=text`, in order).
#[track_caller]
fn assert_rows_warned(rows: &[(&str, &str)], warnings: usize, expected: &[&[&str]]) {
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  for (name, tex) in rows {
    let xml = convert_warned(name, tex, warnings);
    assert_eq!(creators(&xml), want, "{name}: {xml}");
  }
}

/// Each row converts clean to the creators `expected`.
#[track_caller]
pub(super) fn assert_rows(rows: &[(&str, &str)], expected: &[&[&str]]) {
  assert_rows_warned(rows, 0, expected)
}

/// A `tabular` of names, then a hidden `\and`, then `Cy Cole \institute{Univ C}`: Cy Cole is a column of his own, his
/// institute his own. Red on int157a for the macro (`\newcommand\nx{\and}`): "\nx Cy Cole" was read beside Bob Baker's
/// name and the institute under the whole table, Univ C on all three authors.
#[test]
fn eptcs_a_hidden_and_after_a_table_opens_a_column() {
  assert_rows(
    &[
      row!("eptcs_author_table_then_a_macro_held_and"),
      row!("eptcs_author_table_then_a_let_copy_of_and"),
    ],
    &[
      &["Ann Able", "email=ann@a.edu"],
      &["Bob Baker", "email=bob@b.edu"],
      &["Cy Cole", "affiliation=Univ C"],
    ],
  );
}

/// `Ann Able \institute{Univ A}`, then a hidden `\and`, then a `tabular` of names over their e-mails: the second cell
/// opens with the table and is read as names. Red on int157a for the macro: the table was a line under Ann Able, one
/// creator "Ann Able and Bob Baker Cy Cole bob@b.edu cole@c.edu", 1 Warning.
#[test]
fn eptcs_a_cell_after_a_hidden_and_opens_with_a_table() {
  assert_rows(
    &[
      row!("eptcs_author_macro_held_and_then_a_table"),
      row!("eptcs_author_let_copy_of_and_then_a_table"),
    ],
    &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "email=bob@b.edu"],
      &["Cy Cole", "email=cole@c.edu"],
    ],
  );
}

/// No `\institute` or `\email`: `Ann Able`, a hidden `\and`, then a `tabular` of names. The block is the class's
/// because its second cell opens with a table (`eptcs_structured`). Red on int157a for the macro: the gate saw one
/// cell, the kernel's author line made one creator "Ann Able and Bob Baker Cy Cole bob@b.edu cole@c.edu", 1 Warning.
#[test]
fn eptcs_the_gate_sees_a_table_after_a_hidden_and() {
  assert_rows(
    &[
      row!("eptcs_author_macro_held_and_then_a_table_no_lines"),
      row!("eptcs_author_let_copy_of_and_then_a_table_no_lines"),
    ],
    &[&["Ann Able"], &["Bob Baker", "email=bob@b.edu"], &[
      "Cy Cole",
      "email=cole@c.edu",
    ]],
  );
}

/// `\newcommand\aut{Ann Able \and Bob Baker}` then `\author{\aut \institute{Univ X}}`: the `\and`'s tokens print the
/// names too, so no cell is cut there (`ArgSeparator::span` is `None`); the reading cuts the names, the institute under
/// Bob Baker's column only, as pdflatex prints it. Green on int157a too (pinned): the path a cut by tokens must not take.
/// Its names are written from the macro's expansion read in its place (AUT1, `reading::unfold_held_names`); int157a to
/// int167a wrote each by its text, two Warnings ("a name its events don't spell").
#[test]
fn eptcs_a_macro_holding_names_and_the_and_stays_one_cell() {
  assert_rows_warned(
    &[row!("eptcs_author_macro_holding_names_and_the_and")],
    0,
    &[&["Ann Able"], &["Bob Baker", "affiliation=Univ X"]],
  );
}

/// Today's shapes, their whole creator elements (ids aside) as int157a writes them: an `\institute` and an `\email`
/// under each name, a row of names with lines under it and an e-mail list, a leading `tabular` with lines under the
/// table (every name's, `_share=authors`), a paper's own table environment, and the plain cells parted by a macro-held
/// `\and` and by a `\let` copy.
#[test]
fn eptcs_todays_shapes_keep_their_creator_elements() {
  let rows = [
    row!("eptcs_institute_email_cells"),
    row!("eptcs_row_of_names"),
    row!("eptcs_author_tabular"),
    row!("eptcs_author_environment_columns"),
    row!("eptcs_author_cut_at_a_macro_held_and"),
    row!("eptcs_author_let_copy_of_and_parts_the_names"),
  ];
  let found: Vec<String> = rows
    .iter()
    .map(|(name, tex)| {
      format!(
        "== {name}\n{}",
        creators_xml(&convert(name, tex)).join("\n")
      )
    })
    .collect();
  assert_eq!(
    found.join("\n"),
    r##"== eptcs_institute_email_cells
<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Univ A<break/>Country A</contact>
    <contact name="Email: " role="email"><text font="typewriter">ann@a.edu</text></contact>
  </creator>
<creator before="  " role="author">
    <personname>Bob Baker</personname>
    <contact name="Affiliation: " role="affiliation">Univ B</contact>
    <contact name="Affiliation: " role="affiliation">Dept B</contact>
    <contact name="Email: " role="email"><text font="typewriter">bob@b.edu</text></contact>
  </creator>
== eptcs_row_of_names
<creator role="author">
    <personname>Carl Cole</personname>
    <contact name="Affiliation: " role="affiliation">Univ C</contact>
    <contact name="Email: " role="email"><text font="typewriter">cole@c.edu</text></contact>
    <contact name="Affiliation: " role="affiliation">Lab D</contact>
  </creator>
<creator before="  " role="author">
    <personname>Dan Dorn</personname>
    <contact name="Affiliation: " role="affiliation">Univ C</contact>
    <contact name="Email: " role="email"><text font="typewriter">dorn@c.edu</text></contact>
    <contact name="Affiliation: " role="affiliation">Lab D</contact>
    <contact name="Email: " role="email"><text font="typewriter">dorn@d.org</text></contact>
  </creator>
<creator before="  " role="author">
    <personname>Eve Eng</personname>
    <contact name="Affiliation: " role="affiliation">Univ E</contact>
    <contact name="Email: " role="email"><text font="typewriter">e.eng@e.org</text></contact>
  </creator>
<creator before="  " role="author">
    <personname>Fay Fox</personname>
    <contact name="Affiliation: " role="affiliation">Univ E</contact>
    <contact name="Email: " role="email"><text font="typewriter">fx1@e.org</text></contact>
  </creator>
== eptcs_author_tabular
<creator role="author">
    <personname>Ann Able</personname>
    <note class="ltx_note_frontmatter ltx_thanks_funding" role="thanks">Funded.</note>
    <contact name="Email: " role="email"><text font="typewriter" fontsize="80%">ann@a.edu</text></contact>
    <contact name="Affiliation: " role="affiliation"><text fontsize="80%">Dept of Tools</text></contact>
    <contact name="Affiliation: " role="affiliation"><text fontsize="80%">Univ of Here</text></contact>
  </creator>
<creator before="  " role="author">
    <personname>Bob Baker</personname>
    <contact name="Email: " role="email"><text font="typewriter" fontsize="80%">bob@b.edu</text></contact>
    <contact name="Affiliation: " role="affiliation"><text fontsize="80%">Dept of Tools</text></contact>
    <contact name="Affiliation: " role="affiliation"><text fontsize="80%">Univ of Here</text></contact>
  </creator>
== eptcs_author_environment_columns
<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Univ X</contact>
    <contact name="Email: " role="email"><text font="typewriter">ann.able@x.org</text></contact>
  </creator>
<creator before="  " role="author">
    <personname>Bob Baker</personname>
    <contact name="Affiliation: " role="affiliation">Univ X</contact>
    <contact name="Email: " role="email"><text font="typewriter">bob.baker@x.org</text></contact>
  </creator>
<creator before="  " role="author">
    <personname>Carl Cole</personname>
    <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">Equal.</note>
    <contact name="Affiliation: " role="affiliation">Univ X</contact>
    <contact name="Email: " role="email"><text font="typewriter">carl.cole@x.org</text></contact>
  </creator>
== eptcs_author_cut_at_a_macro_held_and
<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Univ A</contact>
  </creator>
<creator before="  " role="author">
    <personname>Bob Baker</personname>
    <contact name="Affiliation: " role="affiliation">Univ B</contact>
  </creator>
== eptcs_author_let_copy_of_and_parts_the_names
<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Univ A</contact>
  </creator>
<creator before="  " role="author">
    <personname>Bob Baker</personname>
    <contact name="Affiliation: " role="affiliation">Univ B</contact>
  </creator>"##
  );
}
