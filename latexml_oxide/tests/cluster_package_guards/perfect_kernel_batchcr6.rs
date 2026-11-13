//! Red/green guards for batch CR6: IEEEtran's `\IEEEauthorblockN` under authblk's `\author` (ieeetran_cls.rs
//! `\lx@IEEE@authblk@author`). The block body is read as the class's own `\author` reads it, one author a name with its
//! grade, and its authors are added after those before them as authblk adds every call's; a labelled call gives each
//! name its label. Through authblk's one creator call the nested `\IEEEauthorblockN` printed nothing there and every
//! author was lost without a Warning (2402.16140). OXIDIZED_DESIGN_DIVERGENCES #987.
use super::perfect_kernel_batchcr2::{convert_clean, elements};

/// `\author{\IEEEauthorblockN{Ann Able, ~\IEEEmembership{Member, ~IEEE}, Bob Baker, … and Cy Coe, …}\thanks{…}\thanks{…}}`
/// under authblk (2402.16140; 1611.02001, 1810.08035, 2409.09417 the same shape; int154u: no creator, 0 Warnings):
/// three authors, each with its grade as a membership contact, the notes the document's, as without authblk. The
/// unlabelled block path. Repro class-bindings/ieeetran_authblk_blockn_names.
#[test]
fn authblk_author_reads_ieee_author_blocks() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_authblk_blockn_names.tex"
  ));
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact role=\"membership\">Member, \u{a0}IEEE</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact role=\"membership\">Senior Member,\n\u{a0}IEEE</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Coe</personname>\n    <contact role=\"membership\">Fellow, \u{a0}IEEE</contact>\n  </creator>",
  ]);
  assert_eq!(elements(&xml, "pubnote"), vec![
    "<pubnote role=\"thanks\">A. Able and B. Baker are with Zebra University.</pubnote>",
    "<pubnote role=\"thanks\">C. Coe is with Yak College.</pubnote>",
  ]);
}

/// Two unlabelled authblk calls of author blocks with their `\IEEEauthorblockA` lines, then `\author[2]{\IEEEauthorblockN{Cy
/// Coe, Dee Dale}}\affil[2]{Univ Z}` (siblings of 2402.16140; int154u: Ann Able and Bob Baker without their lines, a
/// nameless creator holding Univ Z, 1 Warning): every call's authors are kept after those before (the class's own
/// `\author` replaces them), each block line its author's, and the labelled call's names both take `\affil[2]`.
/// Distinct from the guard above: authblk's accumulation and the labelled path (`\lx@IEEE@authorblockN@labelled`).
/// Repro class-bindings/ieeetran_authblk_blockn_calls_accumulate.
#[test]
fn authblk_author_blocks_accumulate_and_take_labels() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_authblk_blockn_calls_accumulate.tex"
  ));
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Y</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Coe</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Dee Dale</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z</contact>\n  </creator>",
  ]);
}
