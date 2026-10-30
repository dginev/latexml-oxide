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

/// A label glued to a footnote symbol in one written piece is the two marks it prints (ruling 1, `ruling1:parted`, a
/// change tag): aa's `\inst{1*}` prints "1∗" — institute 1 and the corresponding author's `*` (2506.20360, its two
/// corresponding authors; `*2` the same, symbol first). Today's old reader matched `1*` against no entry, so the two
/// names had no institute; the reading's owner was untagged, so the writer declined. Distinct from the lexer's parted
/// `\footnotemark` (`fms2a2r5`): here the label's own text holds both.
#[test]
fn a_label_glued_to_a_symbol_is_parted_and_written() {
  assert_reading(
    repro!("aa_inst_glued_symbol_is_parted.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Univ X"],
      &["Cid Cole", "affiliation=Univ Y"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Bob Baker [1,*] | line:names",
      "  a Univ X | api:labelseq ruling1:parted",
      "C Cid Cole [*,2] | line:names",
      "  a Univ Y | api:labelseq ruling1:parted",
    ],
  );
}

/// The writer's write-out operation (`plan::Op::WriteOut`): an `\institute` entry's `\email{\{a,b,c\}@x}` printing
/// several addresses, which the reading writes out (ruling 3, `ruling3:written-out`), each to the author it spells and
/// the rest to the block — today's one contact on every carrier of the entry — is written so: each address a copy of
/// today's contact printing only it (1811.05243, 1812.02463, 2403.14668, 2501.11467: 95 llncs and 25 aa census papers
/// held back for want of it). Siblings on the same path: the shared domain printed inside the braces after the last
/// local part (`\{cid.cole,lab@y.org\}`, 1907.06944; `labels::addresses_in`), and an accented digested name spelled by
/// its base letters (`Ştefan Ciobâcă`'s `stefan.ciobaca@`, 2001.09649; `Resolver::spelled`).
#[test]
fn an_email_group_is_written_out_to_its_spellers() {
  assert_reading(
    repro!("llncs_email_group_written_out.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ X", "email=ann.able@x.org"],
      &[
        "Ştefan Ciobâcă",
        "affiliation=Univ X",
        "email=stefan.ciobaca@x.org",
      ],
      &["Cid Cole", "affiliation=Univ Y", "email=cid.cole@y.org"],
      &["email=info@x.org", "email=lab@y.org"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email ann.able@x.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "C Ştefan Ciobâcă [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email stefan.ciobaca@x.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "C Cid Cole [2] | line:names",
      "  a Univ Y | api:labelseq",
      "  c email cid.cole@y.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "B email info@x.org | api:labelseq api:inner-contact ruling3:written-out owner:block",
      "B email lab@y.org | api:labelseq api:inner-contact ruling3:written-out owner:block",
    ],
  );
}

/// An empty call holds nothing back (`writer::written_resolution`): llncs's split of `Ann Able\inst{1} \and\\ Bob
/// Baker\inst{2}` leaves an empty piece between `\and` and `\\`, a block skipped as empty, which held a written
/// paper back as an unread one (2409.09478: the scorer's R2 `email_spells_other`, its corresponding author's address
/// on another name). Distinct from the write-out: the writer's gate, not its operations.
#[test]
fn an_empty_author_piece_holds_nothing_back() {
  assert_reading(
    repro!("llncs_empty_author_piece_holds_nothing_back.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ X", "email=ann.able@y.org"],
      &["Bob Baker", "affiliation=Univ Y"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email ann.able@y.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "C Bob Baker [2] | line:names",
      "  a Univ Y | api:labelseq",
    ],
  );
}
