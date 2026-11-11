//! Guards for batch P21b (Option C step P2-1b, OD #993): sv_support's `\author` hands its whole name list to the reading
//! in one `\lx@add@author@split` call, where it split it by spelling first (`\lx@splitting`), and the queue's name reader
//! parts that one entry at its kernel `\and`s (`queue_reading.rs` `entry_names`, by `argument_separators`' meaning and
//! position, OD #995), so the `\institute` reader still counts each name (`queued_author_names_spelled`). The repro
//! compiles clean with pdflatex (TL 2025, 2312.14898's svjour3.cls); the guard asserts every creator's name and contacts,
//! in order, and the exact Error and Warning counts.

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

/// An itemized `\institute` list naming each author of a whole queued author entry before its one `\at` (2312.14898's
/// shape: braced names, `\textsuperscript` marks, an `\and \\`, a `\\` before `\at`) is the authors' list: each e-mail goes
/// to the author it spells and the list is shared on the block's nameless creator (#159). Read as one name, the entry
/// spelled no name whole and the list and all three addresses went to Cid Cole alone (P21 without sv_support's split).
/// Two rows, two paths of the part: the kernel `\and` (a constructor, its own token), and a document's
/// `\renewcommand\and` (a macro, which the separator query once gave no position; svjour3 sets its own `\and` at the
/// title, so pdflatex prints the same three names). Distinct from
/// `perfect_kernel_batch98b::svjour3_list_before_at_is_structure`: the witness's own shape, three names, and the part's
/// two paths.
#[test]
fn sv_institute_list_reads_a_whole_author_entry_by_its_names() {
  let expected: Vec<Vec<String>> = [
    &["Ann Able 1", "email=ann@x.edu"][..],
    &["Bob Baker 1", "email=bob@x.edu"],
    &["Cid Cole 2", "email=cid@y.edu"],
    &["affiliation=1 item 1 University of X 2 item 2 University of Y"],
  ]
  .iter()
  .map(|row| row.iter().map(ToString::to_string).collect())
  .collect();
  for (name, tex) in [
    (
      "kernel and",
      repro!("svjour3_institute_list_names_a_whole_author_block"),
    ),
    (
      "document and",
      repro!("svjour3_institute_list_names_a_whole_author_block_document_and"),
    ),
  ] {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert_eq!(creators(&xml), expected, "{name}: {xml}");
    assert_eq!(xml.matches("<creator ").count(), 4, "{name}: {xml}");
    assert_eq!(xml.matches("<personname").count(), 3, "{name}: {xml}");
    assert_eq!(xml.matches("<itemize").count(), 1, "{name}: {xml}");
  }
}
