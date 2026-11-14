//! Guards for P24b (P24's fix round, OXIDIZED_DESIGN_DIVERGENCES #991 (8), #993 P2-1b, #995): a separator's own tokens
//! (`argument_separators`' `ArgSeparator::span`), the one field both callers part an argument at — eptcs's cells
//! (`latexml_contrib/src/eptcs_cls.rs` `cells`) and the queue's name reader (`queue_reading.rs` `entry_names`).
//! An `\end{tabular}` glued to the separator makes no box, so the separator's box covers it (core
//! `token::field_cover_pending_merge`); the span leaves those tokens to the table they close (`separators.rs`
//! `own_run`, the rehearsal's environment steps), so it starts on a unit of the block's top level. One guard per caller:
//! eptcs's glued tails (no space, `%⏎`, `\relax`, a macro holding the `\and`, a `\let` copy), and svjour3's entry parted
//! at a macro holding only the `\and`. Each asserts no Error or Warning and every creator's name and contacts in order.

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchp24::{assert_rows, row},
};

/// A `tabular` of names, then the `\and` glued to its `\end{tabular}`, then a name with an `\institute`: the name after
/// the table opens a column of its own and the institute is its alone. Red on P24 for every row (the separator's span
/// ran from the `\end`, off a unit start, and `cells` dropped the cut: the institute on every author); on int157a the
/// macro row was red too (Univ C on all three: the split by spelling saw one cell), the others green.
#[test]
fn eptcs_a_separator_glued_to_a_table_end_opens_a_column() {
  assert_rows(&[row!("eptcs_author_table_glued_to_and")], &[
    &["Ann Able", "email=ann@a.edu"],
    &["Bob Baker", "affiliation=Univ B"],
  ]);
  assert_rows(
    &[
      row!("eptcs_author_table_comment_then_and"),
      row!("eptcs_author_table_relax_then_and"),
      row!("eptcs_author_table_glued_to_a_macro_held_and"),
      row!("eptcs_author_table_glued_to_a_let_copy_of_and"),
    ],
    &[
      &["Ann Able", "email=ann@a.edu"],
      &["Bob Baker", "email=bob@b.edu"],
      &["Cy Cole", "affiliation=Univ C"],
    ],
  );
}

/// svjour3's whole author list `Ann Able \sep Bob Baker`, `\sep` a macro holding only the `\and`: the queue's name reader
/// parts the entry there, as the reading cuts it, so the `\institute` list naming both before its `\at` is the authors'
/// list — each e-mail to the author it spells, the list on the block's nameless creator (#159). Red on int158a, whose
/// separator position gave a macro none: the entry one name, both addresses and the list on Bob Baker. Distinct from
/// `perfect_kernel_batchp21b::sv_institute_list_reads_a_whole_author_entry_by_its_names` (a written `\and` and a
/// document's `\renewcommand\and`, each one token): a macro call standing for the separator.
#[test]
fn sv_institute_list_reads_an_entry_parted_at_a_macro_held_and() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/author_one_block/svjour3_institute_list_names_a_macro_held_and_entry.tex"
  );
  let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let expected: Vec<Vec<String>> = [
    &["Ann Able", "email=ann@x.edu"][..],
    &["Bob Baker", "email=bob@x.edu"],
    &["affiliation=University of X"],
  ]
  .iter()
  .map(|row| row.iter().map(ToString::to_string).collect())
  .collect();
  assert_eq!(creators(&xml), expected, "{xml}");
  assert_eq!(xml.matches("<creator ").count(), 3, "{xml}");
}
