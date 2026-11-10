//! Red/green guards for batch CR10a: IEEEtran's names blocks (ieeetran_cls.rs `author_block_body`,
//! `without_stop_after_grade`). A names block written inside another's names is spliced into it and read as one block
//! (`unnested_name_blocks`), the old alias `\authorblockN`'s grades read as `\IEEEauthorblockN`'s, and the full stop
//! after a grade written as italic text closes the names as one after `\IEEEmembership{…}` does.
//! OXIDIZED_DESIGN_DIVERGENCES #987.
use super::perfect_kernel_batchcr2::{convert_clean, elements};

/// `\author{\authorblockN{\IEEEauthorblockN{Ann Able, \IEEEmembership{…}, Bob Baker, \IEEEmembership{…} \\ and Cy Coe,
/// \IEEEmembership{…}.} \\ \thanks{…}}}` (1511.03982; int154q: no creator, 0 Warnings — the inner block's author call
/// ran inside the outer's argument and printed nothing): three authors, each with its grade, the note the last
/// author's as in the same block written once. The nesting (on the class's `\author` and authblk's alike, both through
/// `author_block_body`) and the alias's grades (`block_name_memberships`). Repro
/// class-bindings/ieeetran_nested_blockn_names.
#[test]
fn nested_names_block_keeps_every_author() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_nested_blockn_names.tex"
  ));
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact role=\"membership\">Member,\u{a0}IEEE</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact role=\"membership\">Senior Member,\u{a0}IEEE</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Coe</personname>\n    <contact role=\"membership\">Fellow,\u{a0}IEEE</contact>\n    <note class=\"ltx_note_frontmatter ltx_thanks_funding\" role=\"thanks\" xml:id=\"id1\">Zebra funded this.</note>\n  </creator>",
  ]);
}

/// `\IEEEauthorblockN{Ann Able, Bob Baker,\\ Cy Coe, and Dee Dale, {\em Member, IEEE}.}` and `\IEEEauthorblockN{Eve
/// Eads, \textit{Fellow, IEEE}.}` (1810.08035; int154q: "Cy Coe, and Dee Dale, Member, IEEE ." the first two authors'
/// affiliation, Eve Eads's grade cut into a membership "Fellow" and an affiliation "IEEE.", 2 Warnings): five
/// authors, each italic grade its author's membership. Distinct from the guard above: the stop's reading
/// (`without_stop_after_grade`, `grade_text`), not the block structure. Repro
/// class-bindings/ieeetran_blockn_italic_grade_stop.
#[test]
fn italic_grade_before_stop_ends_the_names() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_blockn_italic_grade_stop.tex"
  ));
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Coe</personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Dee Dale</personname>\n    <contact role=\"membership\"><text font=\"italic\" xml:id=\"id1\">Member, IEEE</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Eve Eads</personname>\n    <contact role=\"membership\"><text font=\"italic\" xml:id=\"id2\">Fellow, IEEE</text></contact>\n  </creator>",
  ]);
}
