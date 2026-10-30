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

/// One contact per creator and address (review rev92c S2): the reading gives Ann Able `ann@a.org` from her institute's
/// line and from `\offprints` (0906.2451's S. K. Mathew), and the writer, which wrote each, collapses a reading's
/// identical items of one owner (`plan::wanted`) — today's one contact stays. Distinct from the link: two prints, one
/// address, the writer's collapse rather than the reading's.
#[test]
fn an_address_printed_twice_is_one_contact() {
  assert_reading(
    repro!("aa_address_printed_twice_is_one_contact.tex"),
    Some(&[
      &[
        "Ann Able",
        "affiliation=Univ A",
        "email=ann@a.org",
        "affiliation=Univ B",
      ],
      &["Bob Baker", "affiliation=Univ B"],
    ]),
    &[
      "C Ann Able [1,2] | line:names",
      "  a Univ A | api:labelseq",
      "  a Univ B | api:labelseq",
      "  c email ann@a.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "  c email ann@a.org | api:document-note owner:spelled api:note-spelled ruling3:one-per-print",
      "C Bob Baker [2] | line:names",
      "  a Univ B | api:labelseq",
    ],
  );
}

/// The write-out writes a group only when its contact prints the addresses alone (review rev92c B2): beside a remark
/// (`\{ann,bob\}@a.org (corresponding)`) or a label (`Emails: \{ann,bob\}@a.org`) the words were lost; now the writer
/// declines (`plan::addresses_alone`), today's contact stays whole on both carriers, the reading still writing it out.
#[test]
fn an_email_group_beside_words_stays_whole() {
  for (tex, contact) in [
    (
      repro!("llncs_email_group_beside_text_stays_whole.tex"),
      "email={ann,bob}@a.org (corresponding)",
    ),
    (
      repro!("llncs_email_group_after_a_label_stays_whole.tex"),
      "email=Emails: {ann,bob}@a.org",
    ),
  ] {
    let reading = assert_reading(
      tex,
      Some(&[&["Ann Able", "affiliation=Univ A", contact], &[
        "Bob Baker",
        "affiliation=Univ A",
        contact,
      ]]),
      &[
        "C Ann Able [1] | line:names",
        "  a Univ A | api:labelseq",
        "  c email ann@a.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
        "C Bob Baker [1] | line:names",
        "  a Univ A | api:labelseq",
        "  c email bob@a.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      ],
    );
    assert!(
      reading
        .3
        .as_str()
        .is_some_and(|w| w.starts_with("declined")),
      "{:?}",
      reading.3
    );
  }
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
  assert_reading(
    repro!("aa_sole_institute_address_line_is_the_rows.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ X, City"],
      &["Bob Baker", "affiliation=Univ X, City", "email=bob@x.org"],
      &["Cid Cole", "affiliation=Univ X, City", "email=cid@x.org"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X, City | api:labelseq",
      "C Bob Baker [1,*] | line:names",
      "  a Univ X, City | api:labelseq ruling1:parted",
      "  c email bob@x.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "C Cid Cole [1,*] | line:names",
      "  a Univ X, City | api:labelseq ruling1:parted",
      "  c email cid@x.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
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
      "C Bob Baker [1,2] | line:names",
      "  a Zeta University | api:labelseq ruling1:request",
      "  a Alpha Institute | api:labelseq ruling1:request",
    ],
  );
  assert_reading(
    repro!("llncs_written_out_address_keeps_its_place.tex"),
    Some(&[
      &[
        "Ann Able",
        "affiliation=Univ A",
        "email=ann@a.org",
        "affiliation=Univ B",
        "email=ann@b.org",
      ],
      &["Bob Baker", "affiliation=Univ A", "email=bob@a.org"],
    ]),
    &[
      "C Ann Able [1,2] | line:names",
      "  a Univ A | api:labelseq",
      "  a Univ B | api:labelseq",
      "  c email ann@a.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "  c email ann@b.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "C Bob Baker [1] | line:names",
      "  a Univ A | api:labelseq",
      "  c email bob@a.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
    ],
  );
}

/// A note's item prints below the author block, whatever its call's place (review rev92c r2 B-1; `plan::Place`,
/// `prints_in_note`): aa's `\offprints{A. Able, \email{…}}` is called before `\institute` but prints in the first page's
/// footer, so the writer, which gives the address to A. Able (the note's name) and orders her items, keeps it after her
/// institute — ordered by the calls, it had gone above it (0903.3512's O. Kochukhov; 0902.0930, 1205.5105, 1411.3255,
/// astro-ph0203288, astro-ph0509342, astro-ph0512559, astro-ph0608284, astro-ph0702605). Distinct from
/// `written_items_keep_the_print_order`: the order across the author block and a note, not within the block's calls.
/// The note's text keeps its name. The same for a contact inside a block's `\thanks` (2212.07438).
#[test]
fn a_notes_item_prints_after_the_author_block() {
  let reading = assert_reading(
    repro!("aa_offprints_email_prints_after_the_institutes.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ A, City A", "email=xyz@a.org"],
      &["Bob Baker", "affiliation=Univ B, City B"],
      &[
        "Cid Cole",
        "affiliation=Univ B, City B",
        "affiliation=Univ C, City C",
      ],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ A, City A | api:labelseq",
      "  c email xyz@a.org | api:document-note owner:corresponding api:note-name ruling3:one-per-print",
      "C Bob Baker [2] | line:names",
      "  a Univ B, City B | api:labelseq",
      "C Cid Cole [2,3] | line:names",
      "  a Univ B, City B | api:labelseq",
      "  a Univ C, City C | api:labelseq",
    ],
  );
  assert_eq!(reading.2, ["A. Able,"]);
  assert_eq!(reading.3, "written");
  // (a contact inside a `\thanks` of the author block, which prints in the footer too: 2212.07438's S. Contarini — the
  // writer cuts the hand-marked institute lines and orders her items; by span the note's e-mail went first)
  let reading = assert_reading(
    repro!("aa_thanks_email_prints_after_the_institutes.tex"),
    Some(&[
      &[
        "Ann Able 1",
        "affiliation=1 Univ A, City A",
        "email=ann@a.org",
      ],
      &["Bob Baker 2", "affiliation=2 Univ B, City B"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ A, City A | api:leading-mark api:split-by-marks",
      "  c email ann@a.org | note:adjacent ruling4:inner-contact",
      "C Bob Baker [2] | line:names",
      "  a Univ B, City B | api:leading-mark api:split-by-marks",
      "D note ann@a.org | note:adjacent ruling4:emptied",
    ],
  );
  assert_eq!(reading.3, "written");
}

/// aa's sole institute read as llncs's (review rev92c r2 B-2; OD #932), on the three paths of the shared row
/// (`shared_row::row_email_calls`): an `\email` inside it is a class e-mail call — aa_support's `\email`, Perl's
/// `\lx@add@contact[role=email]`, said as the kernel's `\lx@add@email` (`is_email_command`) — so the address goes to
/// the author it spells and the institute to every author, the comma before the call a separator (1312.1871,
/// 2310.16880: both had been the block's; the llncs twin was right); a `\thanks` inside it prints in the footer, no line
/// of the institute, so its address holds nothing back (`holds_raw_address`; 0705.1240: the institute had been the
/// block's), the note a publication note; a later line the address reader does not part (a remark beside the address)
/// opened by an `e-mail:` label is an e-mail of the block, not a nameless affiliation (astro-ph0007270's `e-mail: gil@…
/// (AGdP)`, review rev92c r2 S-3) — the reading keeps the line in the institute's text, so the writer declines and
/// today's stands.
#[test]
fn aa_sole_institute_reads_as_llncss() {
  let reading = assert_reading(
    repro!("aa_sole_institute_inline_email_is_its_authors.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ A, City A", "email=ann@a.org"],
      &["Bob Baker", "affiliation=Univ A, City A"],
    ]),
    &[
      "C Ann Able [] | line:names",
      "  a Univ A, City A, | api:labelseq api:share:authors",
      "  c email ann@a.org | api:labelseq api:inner-contact api:share:authors owner:spelled",
      "C Bob Baker [] | line:names",
      "  a Univ A, City A, | api:labelseq api:share:authors",
    ],
  );
  assert_eq!(reading.3, "same");
  let reading = assert_reading(
    repro!("aa_sole_institute_thanks_stays_shared.tex"),
    Some(&[&["Ann Able", "affiliation=Univ A, City A"], &[
      "Bob Baker",
      "affiliation=Univ A, City A",
    ]]),
    &[
      "C Ann Able [] | line:names",
      "  a Univ A, City A | api:labelseq api:share:authors",
      "C Bob Baker [] | line:names",
      "  a Univ A, City A | api:labelseq api:share:authors",
    ],
  );
  assert_eq!(reading.2, ["email: ann@a.org"]);
  assert_eq!(reading.3, "same");
  let reading = assert_reading(
    repro!("aa_sole_institute_email_line_is_an_email.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ A, City A"],
      &["Bob Baker", "affiliation=Univ A, City A"],
      &["email=ann@a.org (AA)"],
    ]),
    &[
      "C Ann Able [] | line:names",
      "  a Univ A, City A e-mail: ann@a.org (AA) | api:labelseq api:share:authors",
      "C Bob Baker [] | line:names",
      "  a Univ A, City A e-mail: ann@a.org (AA) | api:labelseq api:share:authors",
    ],
  );
  assert!(
    reading
      .3
      .as_str()
      .is_some_and(|w| w.starts_with("declined")),
    "{:?}",
    reading.3
  );
}

/// A braced group of addresses whose shared domain prints after the brace (`\{bob@mails, cid@mail\}.b.edu`, review
/// rev92c r2 B-3): each address read whole (`labels::addresses_in`; 93a's write-out lost them, read as no address) and
/// the contact read as printing addresses alone (`plan::addresses_alone` took `.b.edu` for a word beside the group, so
/// the writer declined and the whole contact stayed on one carrier — 2407.18899's Zijia Lin lost `linzijia07@`), so
/// it is written out, each address to the author it spells under the institute it prints in. Distinct from
/// `an_email_group_is_written_out_to_its_spellers`: the group's domain outside its braces, after no `@`.
#[test]
fn an_email_group_with_its_domain_after_the_brace() {
  let reading = assert_reading(
    repro!("llncs_email_group_domain_after_the_brace_written_out.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ A", "email=ann.able@a.org"],
      &[
        "Bob Baker",
        "affiliation=Univ A",
        "affiliation=Univ B",
        "email=bob@mails.b.edu",
      ],
      &["Cid Cole", "affiliation=Univ B", "email=cid@mail.b.edu"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ A | api:labelseq",
      "  c email ann.able@a.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "C Bob Baker [1,2] | line:names",
      "  a Univ A | api:labelseq",
      "  a Univ B | api:labelseq",
      "  c email bob@mails.b.edu | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "C Cid Cole [2] | line:names",
      "  a Univ B | api:labelseq",
      "  c email cid@mail.b.edu | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
    ],
  );
  assert_eq!(reading.3, "written");
}
