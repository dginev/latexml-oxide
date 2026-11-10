//! Guards for batch 92c (frontmatter refactor stage 7: aa and llncs written by the observer; `docs/archive/
//! FRONTMATTER_DIGESTED_REFACTOR_2026-11-07.md`, OXIDIZED_DESIGN_DIVERGENCES #837, #920, #922-). Each guard converts its repros
//! in process and with the binary, eager and `--streaming` (`perfect_kernel_batch92a::
//! assert_reading`), and asserts the reading's tuples and the XML's creators whole, and no Error or Warning.
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

/// An empty call holds nothing back (`writer::written_resolution` (deleted)): llncs's split of `Ann Able\inst{1} \and\\ Bob
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

/// An `\href` in an e-mail call is the text it prints, never its target (review rev92c B1): the call's text read
/// `\lx@fm@obs@href{mailto:ann@a.org}{ann@a.org}` whole, two addresses for one print, each a contact the writer wrote
/// (aa 2210.16066, 1907.11241, 2007.06573, 2103.15844; llncs 2405.04152, 2603.13900, every author). Its siblings on
/// the same path (`events::raw_lines`): the printed text in a font (`\texttt{bob@b.org}`), and a link printing a name
/// (`\href{mailto:cid@c.org}{Cid Cole}`), which prints no address.
#[test]
fn an_email_link_is_the_text_it_prints() {
  assert_reading(
    repro!("llncs_email_link_prints_its_text.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ A", "email=ann@a.org"],
      &["Bob Baker", "affiliation=Univ B", "email=bob@b.org"],
      &["Cid Cole", "affiliation=Univ C", "email=Cid Cole"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ A | api:labelseq",
      "  c email ann@a.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "C Bob Baker [2] | line:names",
      "  a Univ B | api:labelseq",
      "  c email bob@b.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "C Cid Cole [3] | line:names",
      "  a Univ C | api:labelseq",
      "  c email Cid Cole | api:labelseq api:inner-contact owner:inner-email owner:spelled",
    ],
  );
}

/// aa's sole institute (review rev92c B3): aa.cls prints one `\institute` unnumbered under the whole author row
/// (aa.cls:886-899), as llncs.cls does, and aa's binding now says so (`\lx@aa@institute@sole`, `_share=authors`): only
/// the class's `\inst{n}` picks it. Hand-typed marks over numbered lines (`{1} Univ A \\ {2} Univ B`, astro-ph0305539:
/// the reading had given the four institutes to J. Daszyńska-Daszkiewicz) leave it the block's — the twin of llncs's
/// 63i (`perfect_kernel_batch63::institute_with_numbered_lines_is_no_marked_list`); unmarked names share it, the
/// address the author it spells (2210.16066); a later line printing addresses alone is an e-mail line of the row in the
/// reading as in today's (`resolver::api::row_address_lines`; 2506.20360, whose `\inst{1*}` authors lost the institute when the
/// two readings of `{cid@x.org}` differed and the writer declined).
#[test]
fn aa_sole_institute_is_the_rows() {
  assert_reading(
    repro!("aa_sole_institute_numbered_lines_is_the_blocks.tex"),
    Some(&[&["Ann Able 1,2"], &["Bob Baker 2"], &[
      "affiliation=1 Univ A, City A 2 Univ B, City B",
    ]]),
    &[
      "C Ann Able [1,2] | line:names",
      "C Bob Baker [2] | line:names",
      "B affiliation 1 Univ A, City A 2 Univ B, City B | api:labelseq label:unrequested owner:block",
    ],
  );
  assert_reading(
    repro!("aa_sole_institute_is_every_authors.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ A, City", "email=ann@a.org"],
      &["Bob Baker", "affiliation=Univ A, City"],
    ]),
    &[
      "C Ann Able [] | line:names",
      "  a Univ A, City | api:labelseq api:share:authors",
      "  c email ann@a.org | api:labelseq api:inner-contact api:share:authors owner:spelled",
      "C Bob Baker [] | line:names",
      "  a Univ A, City | api:labelseq api:share:authors",
    ],
  );
}

/// The items the writer adds go where the reading prints them among their owner's (review rev92c S1), not at the
/// creator's end in the order of their texts: a copy (`Op::Copy`: Bob Baker's institutes 1 and 2, Zeta before Alpha —
/// 2207.06953's Sangyoun Lee, KIST before Yonsei; 2608.05923's S. Mangla and O. M. Smirnov) and a written-out address
/// (`Op::WriteOut`: Ann Able's `ann@a.org` under Univ A, before Univ B — 1812.02463's Ilyass Haloui; 2603.23924's Yuan
/// Gao). `apply::place`, by the reading's print order (`plan::Place`).
#[test]
fn written_items_keep_the_print_order() {
  assert_reading(
    repro!("llncs_copied_items_keep_the_print_order.tex"),
    Some(&[&["Ann Able 1", "affiliation=Zeta University"], &[
      "Bob Baker 1,2",
      "affiliation=Zeta University",
      "affiliation=Alpha Institute",
    ]]),
    &[
      "C Ann Able [1] | line:names",
      "  a Zeta University | api:labelseq ruling1:request",
      // (Z18e: the name after llncs's inline `\and`, which the reading now sees in the line, `sep:and`)
      "C Bob Baker [1,2] | line:names sep:and",
      "  a Zeta University | api:labelseq ruling1:request",
      "  a Alpha Institute | api:labelseq ruling1:request",
    ],
  );
}
