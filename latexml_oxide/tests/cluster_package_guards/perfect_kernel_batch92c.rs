//! Guards for batch 92c (frontmatter refactor stage 7: aa and llncs written by the observer; `docs/parity/
//! FRONTMATTER_DIGESTED_REFACTOR.md`, OXIDIZED_DESIGN_DIVERGENCES #837, #920, #922-). Each guard converts its repros
//! under the shadow switch in process and with the binary, eager and `--streaming` (`perfect_kernel_batch92a::
//! assert_reading`), and asserts the shadow reading's tuples and the XML's creators whole, and no Error or Warning.
//! Every repro compiles clean with pdflatex (TL 2025; the aa repros with 1707.06238's aa.cls, which TeX Live lacks).

use super::perfect_kernel_batch92a::assert_reading;

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// aa's own `\author` replaces (OXIDIZED_DESIGN #920, #922): aa.cls defines none, so the kernel's `\gdef\@author{#1}`
/// keeps the last call's names — pdflatex prints "Bob Baker1 and Cid Cole2", Perl too (inst_support.sty.ltxml:33-35
/// clears on every call); inst_support's `\author`, which aa reached and which OmniBus shares, stacks (OD #53), so
/// Ann Able was kept. The binding's own `\author` also keys aa's blocks `aa.cls` (`ledger::note_binding_load`: the
/// load whose own code made the meaning, after its aa_support's inst_support defined one), read by aa's conventions.
/// The offprints note names no live author: its address is the block's.
#[test]
fn aa_author_replaces_and_keys_aa_blocks() {
  assert_reading(
    repro!("aa_author_given_twice_keeps_the_last.tex"),
    Some(&[
      &["Bob Baker", "affiliation=Univ X"],
      &["Cid Cole", "affiliation=Univ Y"],
      &["email=carl@z.org"],
    ]),
    &[
      "C Bob Baker [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Cid Cole [2] | line:names",
      "  a Univ Y | api:labelseq",
      "B email carl@z.org | api:document-note owner:block ruling3:one-per-print",
    ],
  );
}
