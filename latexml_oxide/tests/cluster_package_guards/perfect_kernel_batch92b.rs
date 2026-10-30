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

/// Piece 2, 85b's `_share` port (llncs class S, 19 papers: 1912.11606, 2302.05745; OD #901): llncs's one `\institute`,
/// no `\and` in it (`\lx@llncs@institute@sole`, `_share=authors`), is printed unnumbered under the whole row of names,
/// every author's, unless the class's `\inst{n}` picks it; the shadow read its number 1, which no name requested, and
/// left it and its addresses on the block. Now every live author's (`api:share:authors`), its e-mail lines placed by
/// FIDELITY_CHARTER §4's chain as today's places them: the author it spells; the entry's addresses one per author in
/// their order (`{u1, u2, u3}@x.org`); the one address left of several authors the corresponding author's; else the
/// block. A mark that is no class request of a number picks nothing — a hand-typed `$^{*}$` or `$^{1}$` (2302.05745,
/// 2607.02596), `\inst{*}` (2207.04574) —; the class's `\inst{1}` picks it (2608.11332, the reading of 92a). Today's XML
/// agrees, and is pinned.
#[test]
fn a_sole_institute_no_inst_picks_is_every_authors() {
  assert_reading(
    repro!("llncs_sole_institute_is_every_authors_in_order.tex"),
    Some(&[
      &["Ann Able ∗", "affiliation=Univ X", "email=u1@x.org"],
      &["Bob Baker ∗", "affiliation=Univ X", "email=u2@x.org"],
      &["Cid Cole", "affiliation=Univ X", "email=u3@x.org"],
    ]),
    &[
      "C Ann Able [*] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "  c email u1@x.org | api:labelseq api:inner-contact ruling3:written-out api:share:authors owner:order",
      "C Bob Baker [*] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "  c email u2@x.org | api:labelseq api:inner-contact ruling3:written-out api:share:authors owner:order",
      "C Cid Cole [] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "  c email u3@x.org | api:labelseq api:inner-contact ruling3:written-out api:share:authors owner:order",
    ],
  );
  assert_reading(
    repro!("llncs_sole_institute_emails_by_the_chain.tex"),
    Some(&[
      &["Ann Able *", "affiliation=Univ X", "email=ann.able@x.org"],
      &["Bob Baker", "affiliation=Univ X", "email=lab@x.org"],
      &["Cid Cole", "affiliation=Univ X"],
    ]),
    &[
      "C Ann Able [*] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "  c email ann.able@x.org | api:labelseq api:inner-contact api:share:authors owner:spelled",
      "C Bob Baker [] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "  c email lab@x.org | api:labelseq api:inner-contact api:share:authors owner:corresponding",
      "  n Corresponding author. | note:adjacent",
      "C Cid Cole [] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
    ],
  );
  assert_reading(
    repro!("llncs_sole_institute_shared_address_is_the_blocks.tex"),
    Some(&[
      &["Ann Able 1", "affiliation=Univ X"],
      &["Bob Baker 1", "affiliation=Univ X"],
      &["email=info@x.org"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "C Bob Baker [1] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "B email info@x.org | api:labelseq api:inner-contact api:share:authors owner:block",
    ],
  );
  assert_reading(
    repro!("llncs_sole_institute_picked_by_inst.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ X", "email=ann.able@x.org"],
      &["Bob Baker"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email ann.able@x.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "C Bob Baker [] | line:names",
    ],
  );
}
