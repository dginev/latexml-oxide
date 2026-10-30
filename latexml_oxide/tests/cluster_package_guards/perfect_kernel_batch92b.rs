//! Guards for batch 92b (frontmatter refactor stage 7, the llncs and aa move onto the observer — the stage-6 gate
//! study's resolver pieces 3-6 —, and stage 4, label keys by their printed text; `docs/parity/
//! FRONTMATTER_DIGESTED_REFACTOR.md`, OXIDIZED_DESIGN_DIVERGENCES #837, #917-#920). Each guard converts its repros
//! under the shadow switch in process and with the binary, eager and `--streaming` (`perfect_kernel_batch92a::
//! assert_reading`), and asserts the shadow reading's tuples whole, the XML's creators whole where today's output is
//! right (else only the same in every mode), and no Error or Warning. Every repro compiles clean with pdflatex (TL 2025;
//! the aa repros with 1707.06238's aa.cls, which TeX Live lacks) and is checked against Perl 0.8.8.

use super::perfect_kernel_batch92a::assert_reading;

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// Piece 1, the gate study's G3 (llncs class G, 27 papers: 1606.03774, 2204.03161; aa 0901.4618, 1401.5494): an
/// `\email` inside a numbered `\institute` entry printing one address is not the entry's label's — llncs and aa print
/// the list once under the names, the address under its institute, and today's (Perl too) copies it onto every name
/// carrying the label. FIDELITY_CHARTER §4's chain places it (`owner:inner-email`, a change tag): the author it spells,
/// any live author (`bob.baker@y.org` under institute 2 is Bob Baker's of institute 1); else the entry's only carrier
/// (`info@y.org`, Bob Baker's); else the carrier marked the corresponding author (`lab@x.org`, Bob Baker's "Corresponding
/// author." note); else the block, once (a group's `info@x.org` under the institute Ann Able and Bob Baker carry). The
/// sibling (2608.29307): a `\\` inside the call prints its addresses one a line — the rehearsal's break was read into
/// the address (`…@x.org\lx@fm@obs@breakbob.baker@…`, one address spelling its first author); now each is written out.
/// Today's XML copies each one-address call onto every carrier, so only the reading is pinned.
#[test]
fn an_entry_email_printing_one_address_is_placed_by_the_chain() {
  assert_reading(repro!("llncs_inner_email_is_its_spellers.tex"), None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X | api:labelseq",
    "  c email ann.able@x.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
    "C Bob Baker [2] | line:names",
    "  a Univ Y | api:labelseq",
    "  c email info@y.org | api:labelseq api:inner-contact owner:inner-email owner:only-carrier",
    "C Cid Cole [1] | line:names",
    "  a Univ X | api:labelseq",
  ]);
  assert_reading(
    repro!("llncs_inner_shared_email_is_the_blocks.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Bob Baker [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email bob.baker@y.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "C Cid Cole [2] | line:names",
      "  a Univ Y | api:labelseq",
      "  c email cid@y.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "B email info@x.org | api:labelseq api:inner-contact owner:inner-email owner:block",
    ],
  );
  assert_reading(
    repro!("llncs_inner_email_is_the_corresponding_carriers.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Bob Baker [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email lab@x.org | api:labelseq api:inner-contact owner:inner-email owner:corresponding",
      "  n Corresponding author. | note:adjacent",
      "C Cid Cole [2] | line:names",
      "  a Univ Y | api:labelseq",
    ],
  );
  assert_reading(repro!("aa_inner_email_is_its_spellers.tex"), None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X | api:labelseq",
    "C Bob Baker [1] | line:names",
    "  a Univ X | api:labelseq",
    "  c email bob.baker@x.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
    "C Cid Cole [2] | line:names",
    "  a Univ Y | api:labelseq",
  ]);
  assert_reading(
    repro!("llncs_inner_email_lines_are_each_written_out.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email ann.able@x.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "C Bob Baker [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email bob.baker@x.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "C Cid Cole [2] | line:names",
      "  a Univ Y | api:labelseq",
      "B email office@x.org | api:labelseq api:inner-contact ruling3:written-out owner:block",
    ],
  );
}
