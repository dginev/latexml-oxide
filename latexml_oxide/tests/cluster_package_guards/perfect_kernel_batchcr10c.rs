//! Red/green guards for batch CR10c: IEEEtran's names blocks (ieeetran_cls.rs `author_block_body`). The old block
//! names read as the ones they still mean, once, before any step (`with_block_names`), and an outer names block left
//! unclosed around its own affiliation block and the next author's blocks closed before that affiliation block
//! (`unnested_name_blocks`, `affiliation_before_names`). OXIDIZED_DESIGN_DIVERGENCES #987.
use super::perfect_kernel_batchcr2::{convert_clean, elements};

/// `\author{\IEEEauthorblockN{Ann Able\\ \IEEEauthorblockA{Zebra University\\ ann@z.edu}\\ \IEEEauthorblockN{Bob
/// Baker} \IEEEauthorblockA{Yak Institute\\ bob@y.edu}}}` (2605.20445; int154o: "Bob Baker" Ann Able's affiliation
/// and his lines hers, 2 Warnings — spliced in as a wrapper's inner block): two authors, each with the lines under
/// the name as pdflatex stacks them. Distinct from `perfect_kernel_batchcr10a::nested_names_block_keeps_every_author`:
/// the outer block holds its own affiliation block, so it is closed there, not spliced. Repro
/// class-bindings/ieeetran_unclosed_names_block_before_affiliation.
#[test]
fn unclosed_names_block_closes_before_its_affiliation() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_unclosed_names_block_before_affiliation.tex"
  ));
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Zebra University</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\">ann@z.edu</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Yak Institute</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\">bob@y.edu</contact>\n  </creator>",
  ]);
}

/// `\author[2]{\authorblockN{Cy Coe, Dee Dale}}` with `\affil[2]{Univ Z}` under authblk, after an unlabelled
/// `\IEEEauthorblockN` call (int154o: Univ Z a nameless creator's, the names without it, 0 Warnings — the labelled
/// path rewrote `\IEEEauthorblockN` only): each name takes the label and Univ Z. Distinct from
/// `perfect_kernel_batchcr6::authblk_author_blocks_accumulate_and_take_labels`: the old name, read as the block name
/// it means before the labelled path runs. Repro class-bindings/ieeetran_authblk_labelled_old_block_name.
#[test]
fn old_block_name_takes_the_label() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_authblk_labelled_old_block_name.tex"
  ));
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Coe</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Dee Dale</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z</contact>\n  </creator>",
  ]);
}
