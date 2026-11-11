//! Guards for batch P21d (P21's regression, OD #993 P2-1): with OmniBus's `\author` list one block the reading cuts at
//! its `\and`s, a column whose name a contact call follows (`\and Bob Baker\institute{Univ B}`) read as the shared
//! affiliation of every name before it, its text "Bob Baker Univ B" an institution: the reading's shared-column rule
//! (`resolver/blocks.rs` `shared_column`) reads the column's own ink (`column_own_text`), and a column of contacts alone
//! on those contacts. Each repro compiles clean with pdflatex (TL 2025, `nosuchclass` as `article`, its
//! `\providecommand`s setting each contact under its name); the guard asserts every creator's name and contacts, in
//! order, and the exact Error and Warning counts.

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

/// A column's name with the contact its call adds is that name's, under a plain `\and`, after a `tabular` name glued
/// to the `\and` (revP24b's P0) and as the third of three columns (red on int158a: one creator or two, the column's
/// name an affiliation of the names before; nucl-th0308018's `\address`); a column printing only a contact stays every
/// name's (green on int158a too). The one Warning: OmniBus's missing class binding.
#[test]
fn a_column_is_read_on_its_own_ink_not_its_contacts() {
  for (name, tex, expected) in [
    (
      "plain and",
      repro!("omnibus_author_name_with_institute_after_and"),
      &[&["Ann Able"][..], &["Bob Baker", "affiliation=Univ B"]][..],
    ),
    (
      "table glued to the and",
      repro!("omnibus_author_table_glued_to_and_name_with_institute"),
      &[&["Ann Able", "email=ann@a.edu"][..], &[
        "Bob Baker",
        "affiliation=Univ B",
      ]][..],
    ),
    (
      "last of three",
      repro!("omnibus_author_last_of_three_with_address"),
      &[&["Ann Able", "affiliation=Univ A"][..], &["Bob Baker"], &[
        "Cy Cole",
        "address=Univ C",
      ]][..],
    ),
    (
      "contacts alone",
      repro!("omnibus_author_column_of_contacts_alone_is_shared"),
      &[&["Ann Able", "affiliation=Univ AB"][..], &[
        "Bob Baker",
        "affiliation=Univ AB",
      ]][..],
    ),
  ] {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 1),
      "{name}: {log}"
    );
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|row| row.iter().map(ToString::to_string).collect())
      .collect();
    assert_eq!(creators(&xml), expected, "{name}");
  }
}
