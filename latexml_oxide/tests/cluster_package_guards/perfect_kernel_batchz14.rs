//! Guards for Z14 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, Route D): the reading's repairs before the flip. Each asserts the whole reading (the
//! shadow's tuples, names and items with their roles) and the output, the writer's where it writes. Each guard is a different code path: `a_role_heading_…` a later line's
//! test and a name's shape, `a_marked_list_…` the rehearsal's binding of `\lx@add@affiliation@marked` and an in-block line of addresses, `a_speakers_name_…` its binding of
//! `\lx@add@creator`, `a_collaboration_in_a_call_…` a names line with no name before it in its block, `a_line_opening_with_and_…` and `an_address_is_no_piece_…` the later line's
//! other two tests, `an_address_groups_braces_…` the address reader, `a_note_inside_a_link_…` the lexer of a link. Z14r2's (review revZ14): `an_entrys_addresses_are_cut_…` the
//! writer's give-back and its made line's cut, with the read-back's `given_back`; `an_address_group_in_a_names_row_…` the names line's address reader; `an_elided_particle_…` the
//! shared name shape; `a_role_heading_over_words_…` the later line's heading.

use latexml::util::test::warning_count;
use serde_json::Value;

use super::{
  perfect_kernel_batch46::error_count,
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchfms2a2::tuples,
  perfect_kernel_batchz1::{blocks, dump, run},
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The reading of a dump, one line per item without the rule that decided it: `C <name> [<requests>]`, its `a <text>`, `c <role> <text>`, `n <text>`; `B <role> <text>` the block's.
fn reading(dump: &Value) -> Vec<String> {
  tuples(&dump["resolved"])
    .into_iter()
    .map(|item| {
      item
        .rsplit_once(" | ")
        .map_or(item.as_str(), |(item, _)| item)
        .to_string()
    })
    .collect()
}

/// A dump's blocks hold no error, and none of their events is held.
fn nothing_held(dump: &Value) {
  assert_eq!(dump["summary"]["held"], 0, "{dump}");
  for block in blocks(dump) {
    assert_eq!(block["errors"], 0, "{block}");
  }
}

/// `creators` as owned strings, for a table's literal.
fn creators_are(xml: &str, expected: &[&[&str]]) {
  let expected: Vec<Vec<String>> = expected
    .iter()
    .map(|creator| creator.iter().map(ToString::to_string).collect())
    .collect();
  assert_eq!(creators(xml), expected, "{xml}");
}

/// A role printed over names, `{\bf Authors:} F.~Arleo$^a$, …`, opens a row of names, a person's elided particle part of the name's shape (hep-ph0311131, 2205.07715): the rows
/// were the first row's affiliations, every mark unanswered. The heading stays in the name, as written.
#[test]
fn a_role_heading_opens_a_row_of_names() {
  let (xml, dump) = dump(repro!("z14_role_heading_names_row"));
  assert_eq!(reading(&dump), [
    "C Conveners: A. Able []",
    "C B. Baker []",
    "C Editor: A. Able []",
    "C Authors: A. Able [a]",
    "  a Univ A, Town A",
    "C B. Baker [b]",
    "  a Univ B, Town B",
    "C D. d’Enterria [a,b]",
    "  a Univ A, Town A",
    "  a Univ B, Town B",
  ]);
  creators_are(&xml, &[
    &["Conveners : A. Able"],
    &["B. Baker"],
    &["Editor : A. Able"],
    &["Authors: A. Able", "affiliation=Univ A, Town A"],
    &["B. Baker", "affiliation=Univ B, Town B"],
    &[
      "D. d’Enterria",
      "affiliation=Univ A, Town A",
      "affiliation=Univ B, Town B",
    ],
  ]);
}

/// A binding's marked list in a block is read as digested: its marks and lines from the events, its options undigested, no error held on the block (2603.28168, 1405.3507); a
/// block of addresses alone is e-mails by the owner chain, whatever command made it, no affiliation of the last `\and` column's name; an entry's own line of address goes out of it.
#[test]
fn a_marked_list_in_a_plain_author_is_read_digested() {
  let (xml, shadow) = dump(repro!("z14_ieee_marked_list_in_plain_author"));
  nothing_held(&shadow);
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Univ A, Town A",
    "  c email ann.able@a.org",
    "C Bob Baker [2]",
    "  a Univ B, Town B",
    "  c email bob.baker@b.org",
    "C Cat Cole [1,*]",
    "  a Univ A, Town A",
    "  c email cat.cole@a.org",
    "  n Cat Cole is the corresponding author.",
  ]);
  creators_are(&xml, &[
    &[
      "Ann Able",
      "affiliation=Univ A, Town A",
      "email=ann.able@a.org",
    ],
    &[
      "Bob Baker",
      "affiliation=Univ B, Town B",
      "email=bob.baker@b.org",
    ],
    &[
      "Cat Cole",
      "affiliation=Univ A, Town A",
      "email=cat.cole@a.org",
    ],
  ]);
  let (xml, shadow) = dump(repro!("z14_ieee_marked_entry_with_address_lines"));
  nothing_held(&shadow);
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Dept X, Univ X, Town X",
    "  c email ann@x.org",
    "C Bob Baker [2]",
    "  a Dept Y, Univ Y, Town Y",
    "  c email bob@y.org",
  ]);
  creators_are(&xml, &[
    &[
      "Ann Able",
      "affiliation=Dept X, Univ X, Town X",
      "email=ann@x.org",
    ],
    &[
      "Bob Baker",
      "affiliation=Dept Y, Univ Y, Town Y",
      "email=bob@y.org",
    ],
  ]);
}

/// A binding's own creator in a block prints its name there, as PoS.cls:681's `\speaker` does (1412.5940, 1509.01480, 1211.4334): the mark after it its request, no line's lead.
#[test]
fn a_speakers_name_prints_in_its_block() {
  let (xml, shadow) = dump(repro!("z14_speaker_prints_its_name_in_the_block"));
  assert_eq!(reading(&shadow), [
    "C A. Able [1]",
    "  a Univ A, Town A",
    "  c email able@a.org",
    "C B. Baker [2]",
    "  a Univ B, Town B",
  ]);
  creators_are(&xml, &[
    &["A. Able", "affiliation=Univ A, Town A", "email=able@a.org"],
    &["B. Baker", "affiliation=Univ B, Town B"],
  ]);
}

/// A call of its own naming a collaboration is no name, as the PDF prints it under the names: a contact on the names since the last one (1201.5726, 1301.6619; an honest
/// `creator_not_a_name` under the emitter).
#[test]
fn a_collaboration_in_a_call_of_its_own_covers_the_names_before() {
  let (xml, shadow) = dump(repro!("z14_collaboration_in_a_call_of_its_own"));
  assert_eq!(reading(&shadow), [
    "C A. Able [cor:cor]",
    "  a Univ A, Town A",
    "  c collaboration on behalf of the KM3NeT consortium",
    "  c correspondent Corresponding author",
    "C B. Baker []",
    "  a Univ A, Town A",
    "  c collaboration on behalf of the KM3NeT consortium",
  ]);
  creators_are(&xml, &[
    &[
      "A. Able",
      "collaboration=on behalf of the KM3NeT consortium",
      "address=Univ A, Town A",
      "correspondent=Corresponding author",
    ],
    &[
      "B. Baker",
      "collaboration=on behalf of the KM3NeT consortium",
      "address=Univ A, Town A",
    ],
  ]);
  // (a replaced call's names print nothing: the kernel's second `\\author` is the only one printed, so it is no one's collaboration; the Error on it the honest one)
  let (log, _, shadow) = run(
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able}\n\\author{on behalf of the KM3NeT consortium}\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(reading(&shadow), [
    "C on behalf of the KM3NeT consortium []"
  ]);
}

/// A line under the names opening with "and" goes on with them (2007.10022), as `names_list_continues` reads it.
#[test]
fn a_line_opening_with_and_goes_on_with_the_names() {
  let (xml, shadow) = dump(repro!("z14_name_line_opening_with_and"));
  assert_eq!(reading(&shadow), [
    "C Ann Able []",
    "  a University of Town",
    "C Bob Baker []",
    "  a University of Town",
  ]);
  creators_are(&xml, &[&["Ann Able", "affiliation=University of Town"], &[
    "Bob Baker",
    "affiliation=University of Town",
  ]]);
}

/// An address a later row's run prints is no piece of its names (2312.14886: `\email` a font switch): every row its person.
#[test]
fn an_address_is_no_piece_of_a_later_names_row() {
  let (xml, shadow) = dump(repro!("z14_address_in_a_later_names_row"));
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Univ A, Town A",
    "  c email ann.able@a.org",
    "C Bob Baker [2]",
    "  a Univ B, Town B",
    "  c email bob.baker@b.org",
  ]);
  creators_are(&xml, &[
    &[
      "Ann Able",
      "email=ann.able@a.org",
      "affiliation=Univ A, Town A",
    ],
    &[
      "Bob Baker",
      "email=bob.baker@b.org",
      "affiliation=Univ B, Town B",
    ],
  ]);
}

/// An address group's braces set in math, as a verbatim argument holds them (1106.2694): each address its author's, not one of the block.
#[test]
fn an_address_groups_braces_may_be_set_in_math() {
  let (xml, shadow) = dump(repro!("z14_address_group_braces_in_math"));
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Univ A, Town A",
    "  c email ann.able@a.org",
    "C Bob Baker [1]",
    "  a Univ A, Town A",
    "  c email bob.baker@a.org",
    "C Cat Cole [2]",
    "  a Univ B, Town B",
    "  c email cole@b.org",
  ]);
  creators_are(&xml, &[
    &[
      "Ann Able",
      "affiliation=Univ A, Town A",
      "email=ann.able@a.org",
    ],
    &[
      "Bob Baker",
      "affiliation=Univ A, Town A",
      "email=bob.baker@a.org",
    ],
    &["Cat Cole", "affiliation=Univ B, Town B", "email=cole@b.org"],
  ]);
}

/// A note inside a link is an event of its own after it (2507.22485): the link's text its name, its ORCID its contact.
#[test]
fn a_note_inside_a_link_is_an_event_of_its_own() {
  let (xml, shadow) = dump(repro!("z14_note_inside_an_orcid_link"));
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Univ A, Town A",
    "  c orcid 0000-0003-4578-5780",
    "  n ann@a.org",
  ]);
  creators_are(&xml, &[&["Ann Able", "affiliation=Univ A, Town A"]]);
}

/// A row whose pieces each carry a mark, most of them names and none an institution, is a row of names, its odd piece one too (2409.07809's `Aaron (Ari) Bornstein`).
#[test]
fn an_odd_piece_stays_in_its_marked_row() {
  let (xml, shadow) = dump(repro!("z14_odd_piece_in_a_marked_row"));
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Univ A, Town A",
    "C Bob Baker [2]",
    "  a Univ B, Town B",
    "C Cat Cole [1]",
    "  a Univ A, Town A",
    "C Dan (Danny) Dow [2]",
    "  a Univ B, Town B",
    "C Eve Ely [1]",
    "  a Univ A, Town A",
  ]);
  creators_are(&xml, &[
    &["Ann Able", "affiliation=Univ A, Town A"],
    &["Bob Baker", "affiliation=Univ B, Town B"],
    &["Cat Cole", "affiliation=Univ A, Town A"],
    &["Dan (Danny) Dow", "affiliation=Univ B, Town B"],
    &["Eve Ely", "affiliation=Univ A, Town A"],
  ]);
}

/// The ACM SIG classes' `\alignauthor`, a raw class's tabular juggle, is a column in the rehearsal (1503.06772, 1305.7403, 1502.04662): nothing held, each column its creator, as
/// `author_block::separators` splits it; a binding that declares it keeps what it prints (OmniBus, `perfect_kernel_batchz2::a_break_a_binding_declares_stands_beside_the_events`).
#[test]
fn alignauthor_is_a_column_of_the_rehearsal() {
  let (xml, shadow) = dump(repro!("z14_alignauthor_column_of_a_raw_class"));
  nothing_held(&shadow);
  assert_eq!(reading(&shadow), [
    "C Ann Able []",
    "  a Univ A",
    "  c email ann.able@a.org",
    "C Bob Baker []",
    "  a Univ B",
    "  c email bob.baker@b.org",
    "C Cat Cole []",
    "  a Univ C",
    "  a Lab C",
    "  c email cat.cole@c.org",
  ]);
  creators_are(&xml, &[
    &["Ann Able", "affiliation=Univ A", "email=ann.able@a.org"],
    &["Bob Baker", "affiliation=Univ B", "email=bob.baker@b.org"],
    &[
      "Cat Cole",
      "affiliation=Univ C",
      "affiliation=Lab C",
      "email=cat.cole@c.org",
    ],
  ]);
}

/// A line of addresses alone leaves a labelled entry open (1209.0869, 2606.23976, 0910.1386, 1411.2744): the unmarked line after it continues the entry, as today's one contact,
/// and is no affiliation of every name; the address its owner's by the chain.
#[test]
fn a_line_of_addresses_leaves_the_entry_open() {
  let (xml, shadow) = dump(repro!("z14_line_after_a_line_of_addresses"));
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Univ A",
    "C Bob Baker [2]",
    "  a Univ B To appear in Some Journal",
    "  c email bob.baker@b.org",
  ]);
  creators_are(&xml, &[&["Ann Able", "affiliation=Univ A"], &[
    "Bob Baker",
    "affiliation=Univ B To appear in Some Journal",
    "email=bob.baker@b.org",
  ]]);
}

/// A contact a link in it prints, the constructor's ink empty: its text the link's address (2603.01171; the e-mail was lost from the reading).
#[test]
fn a_url_in_a_contact_call_is_its_text() {
  let (xml, shadow) = dump(repro!("z14_url_address_in_a_contact_call"));
  assert_eq!(reading(&shadow), [
    "C Ann Able []",
    "  a Dept X Univ X",
    "  c email ann.able@x.no",
  ]);
  creators_are(&xml, &[&[
    "Ann Able",
    "affiliation=Dept X Univ X",
    "email=ann.able@x.no",
  ]]);
}

/// A tex document of `class` printing `author` as its `\author`, for a table's rows.
fn document(class: &str, author: &str) -> String {
  format!(
    "\\documentclass{{{class}}}\n\\begin{{document}}\n\\title{{T}}\n\\author{{{author}}}\n\\maketitle\nBody.\n\\end{{document}}\n"
  )
}

/// An entry's addresses the reading took out as e-mails, today's one item of the block's lines printing them (2310.19837): its line made from its tokens and cut,
/// its addresses their authors' contacts — the give-back had dropped the contacts and left the addresses in the last affiliation, read back `written`, no difference. One
/// address on its entry's own line is the same path; an address that spells no one stays in its entry in the reading, and the writer makes no line printing it (today's
/// kept: a later pass gives it to its carriers). A give-back the writer applies is read back against the reading as resolved (`given_back`).
#[test]
fn an_entrys_addresses_are_cut_where_its_line_is_made() {
  let (xml, shadow) = dump(repro!("z14r2_entry_addresses_cut_under_a_names_row"));
  assert_eq!(reading(&shadow), [
    "C Ann Able [†]",
    "  a Univ A",
    "  c email ann@a.org",
    "C Bob Baker [‡]",
    "  a Univ B",
    "  c email bob@b.org",
  ]);
  assert_eq!(shadow["writer"], "written", "{shadow}");
  assert_eq!(shadow["readback"]["difference"], Value::Null, "{shadow}");
  assert_eq!(shadow["readback"]["given_back"], Value::Null, "{shadow}");
  creators_are(&xml, &[
    &["Ann Able †", "affiliation=Univ A", "email=ann@a.org"],
    &["Bob Baker ‡", "affiliation=Univ B", "email=bob@b.org"],
  ]);
  for (author, writer, expected) in [
    (
      r"\IEEEauthorblockN{Ann Able$^\dagger$, Bob Baker$^\ddagger$ \IEEEauthorblockA{$^\dagger$Univ A\\ $^\ddagger$Univ B, bob@b.org}}",
      "written",
      &[&["Ann Able †", "affiliation=Univ A"][..], &[
        "Bob Baker ‡",
        "affiliation=Univ B",
        "email=bob@b.org",
      ]][..],
    ),
    (
      r"\IEEEauthorblockN{Ann Able$^\dagger$, Bob Baker$^\ddagger$ \IEEEauthorblockA{$^\dagger$Univ A\\ $^\ddagger$Univ B\\ Email: x@a.org, y@b.org}}",
      "declined",
      &[
        &[
          "Ann Able †",
          "affiliation=† Univ A ‡ Univ B",
          "email=x@a.org",
          "email=y@b.org",
        ][..],
        &[
          "Bob Baker ‡",
          "affiliation=† Univ A ‡ Univ B",
          "email=x@a.org",
          "email=y@b.org",
        ],
      ][..],
    ),
  ] {
    let (xml, shadow) = dump(&document("IEEEtran", author));
    assert!(
      shadow["writer"].as_str().unwrap_or("").starts_with(writer),
      "{author}: {shadow}"
    );
    creators_are(&xml, expected);
  }
  // (a creator the writer conceded to today's, the reading's names not written: read back against the reading as resolved)
  let (_, shadow) = dump(&document("article", r"Inst. d'Astrophysique, Univ A"));
  assert_eq!(shadow["readback"]["difference"], Value::Null, "{shadow}");
  assert_eq!(
    shadow["readback"]["given_back"], "the creators' names",
    "{shadow}"
  );
}

/// A group of addresses in a names row whose printed braces hold commas is read whole, as an address reader reads it (2003.02512): no `E-mails: {ann` and `bob}@a.org`
/// among the people, the one no one spells the last author's by elimination. The writer declines (what it would write drops the label word), today's merged creator and its honest Error kept; set in a font it is written.
#[test]
fn an_address_group_in_a_names_row_is_read_whole() {
  let (log, xml, shadow) = run(repro!("z14r2_address_group_in_a_names_row"));
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("Error:frontmatter:merged_creators"), "{log}");
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Univ A",
    "  c email ann@a.org",
    "C Bob Baker [1]",
    "  a Univ A",
    "  c email bob@a.org",
    "C Cat Cole [2]",
    "  a Univ B",
    "  c email dan@b.org",
  ]);
  assert!(
    shadow["writer"]
      .as_str()
      .unwrap_or("")
      .starts_with("declined"),
    "{shadow}"
  );
  assert_eq!(creators(&xml).len(), 1, "{xml}");
  let (xml, shadow) = dump(&document(
    "IEEEtran",
    r"\IEEEauthorblockN{Ann Able\IEEEauthorrefmark{1}, Bob Baker\IEEEauthorrefmark{2} \IEEEauthorblockA{\IEEEauthorrefmark{1} Univ A} \IEEEauthorblockA{\IEEEauthorrefmark{2} Univ B} \texttt{\{ann, bob\}@a.org}}",
  ));
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Univ A",
    "  c email ann@a.org",
    "C Bob Baker [2]",
    "  a Univ B",
    "  c email bob@a.org",
  ]);
  creators_are(&xml, &[
    &["Ann Able", "email=ann@a.org", "affiliation=Univ A"],
    &["Bob Baker", "email=bob@a.org", "affiliation=Univ B"],
  ]);
}

/// A particle elided onto a capital is a surname's only right after an initial (hep-ph0311131's `D.~d'Enterria`): the shared shape decides today's reader too, and an
/// institution or a place elides as well — `C. Vignat, Universit\'e d'Orsay` one creator as written (1009.2396), a `Villeneuve d'Ascq` row no person in the reading.
#[test]
fn an_elided_particle_is_a_surnames_only_after_an_initial() {
  let (xml, _) = dump(repro!("z14r2_elided_particle_only_after_an_initial"));
  creators_are(&xml, &[&["C. Vignat, Université d’Orsay"]]);
  let (_, shadow) = dump(&document(
    "article",
    r"Ann Able\\ Univ A\\ \\ Villeneuve d'Ascq",
  ));
  assert_eq!(reading(&shadow), [
    "C Ann Able []",
    "  a Univ A",
    "  a Villeneuve d’Ascq",
  ]);
  let (xml, _) = dump(&document("article", r"D. d'Enterria, A. Able"));
  creators_are(&xml, &[&["D. d’Enterria"], &["A. Able"]]);
}

/// A role heading opens a row of names only where each piece shows a person — a name's cue or its mark (hep-ph0311131): a heading over words is a line of the
/// block, as before Z14 (review revZ14's REDs: `Editors: Springer Nature` was a person).
#[test]
fn a_role_heading_over_words_opens_no_names() {
  let (xml, shadow) = dump(repro!("z14r2_role_heading_over_words"));
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Editors: Springer Nature",
    "  a Univ A",
  ]);
  creators_are(&xml, &[&[
    "Ann Able",
    "affiliation=Editors: Springer Nature",
    "affiliation=Univ A",
  ]]);
  for (author, expected) in [
    (
      r"Ann Able\\ Author: Corresponding Author\\ Univ A",
      &[
        "C Ann Able []",
        "  a Author: Corresponding Author",
        "  a Univ A",
      ][..],
    ),
    (r"Ann Able\\ Editor: Physical Review", &[
      "C Ann Able []",
      "  a Editor: Physical Review",
    ]),
    (
      r"Ann Able\\ Contributors: Equal Contribution, Data Curation",
      &[
        "C Ann Able []",
        "  a Contributors: Equal Contribution, Data Curation",
      ],
    ),
  ] {
    let (_, shadow) = dump(&document("article", author));
    assert_eq!(reading(&shadow), expected, "{author}");
  }
}

/// RED (review revZ14): a mark past a marked list's leading run is flattened into its text, `Town, USA\\ *ann@x.org` reading `USA1ann@x.org` (2411.03172, 2510.06608,
/// 2602.11758): each mark should be an event, the institute both authors', each address its author's.
#[test]
#[ignore = "RED: a marked list's mark past its leading run is flattened into the text (events/mod.rs)"]
fn a_mark_past_a_marked_lists_leading_run_is_an_event_red() {
  let (_, shadow) = dump(repro!("z14r2_marked_list_mark_past_the_leading_run_red"));
  assert_eq!(reading(&shadow), [
    "C Ann Able [1]",
    "  a Univ X, Town, USA",
    "  c email ann@x.org",
    "C Bob Baker [2]",
    "  a Univ X, Town, USA",
    "  c email bob@x.org",
  ]);
}
