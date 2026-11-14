//! Guards for Z14 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, Route D): the reading's repairs before the flip. Each asserts the whole reading (the
//! reading's tuples, names and items with their roles) and the output, the writer's where it writes. Each guard is a different code path: `a_role_heading_…` a later line's
//! test and a name's shape, `a_marked_list_…` the rehearsal's binding of `\lx@add@affiliation@marked` and an in-block line of addresses, `a_speakers_name_…` its binding of
//! `\lx@add@creator`, `a_collaboration_in_a_call_…` a names line with no name before it in its block, `a_line_opening_with_and_…` and `an_address_is_no_piece_…` the later line's
//! other two tests, `an_address_groups_braces_…` the address reader, `a_note_inside_a_link_…` the lexer of a link. Z14r2's (review revZ14): `an_entrys_addresses_are_cut_…` the
//! writer's give-back and its made line's cut, with the read-back's `given_back`; `an_address_group_in_a_names_row_…` the names line's address reader; `an_elided_particle_…` the
//! shared name shape; `a_role_heading_over_words_…` the later line's heading.
//! Z14r3's (the bundle's full A/B): `an_address_glued_to_a_dash_…` the names row's address cut; `an_api_calls_pieces_join_…` the take-out's join (`plan::joins_entries`);
//! `an_entry_whose_cut_fails_…` the writer's fail-safe; `a_line_keeping_an_address_…` the block lines' cut (`insert::entry_line_addresses`); `an_only_carriers_address_…`
//! the give-back's weak owner.

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

/// A binding's marked list in a block is read as digested: its marks and lines from the events, its options undigested, no error held on the block (2603.28168, 1405.3507); a
/// block of addresses alone is e-mails by the owner chain, whatever command made it, no affiliation of the last `\and` column's name; an entry's own line of address goes out of it.
#[test]
fn a_marked_list_in_a_plain_author_is_read_digested() {
  let (xml, json) = dump(repro!("z14_ieee_marked_list_in_plain_author"));
  nothing_held(&json);
  assert_eq!(reading(&json), [
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
  let (_, json) = dump(repro!("z14_ieee_marked_entry_with_address_lines"));
  nothing_held(&json);
  assert_eq!(reading(&json), [
    "C Ann Able [1]",
    "  a Dept X, Univ X, Town X",
    "  c email ann@x.org",
    "C Bob Baker [2]",
    "  a Dept Y, Univ Y, Town Y",
    "  c email bob@y.org",
  ]);
}

/// A binding's own creator in a block prints its name there, as PoS.cls:681's `\speaker` does (1412.5940, 1509.01480, 1211.4334): the mark after it its request, no line's lead.
#[test]
fn a_speakers_name_prints_in_its_block() {
  let (xml, json) = dump(repro!("z14_speaker_prints_its_name_in_the_block"));
  assert_eq!(reading(&json), [
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
/// `creator_not_a_name` under the emitter); alone, it is the creator (SBX1).
#[test]
fn a_collaboration_in_a_call_of_its_own_covers_the_names_before() {
  let (xml, json) = dump(repro!("z14_collaboration_in_a_call_of_its_own"));
  assert_eq!(reading(&json), [
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
  // (a replaced call's names print nothing: the kernel's second `\\author` is the only one printed, so it is no one's collaboration;
  // its creator the group the PDF prints as the author, no Error: SBX1, as 2605.24212's `for the PAROS … Network`)
  let (log, _, json) = run(
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able}\n\\author{on behalf of the KM3NeT consortium}\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(reading(&json), ["C on behalf of the KM3NeT consortium []"]);
}

/// A line under the names opening with "and" goes on with them (2007.10022), as `names_list_continues` reads it.
#[test]
fn a_line_opening_with_and_goes_on_with_the_names() {
  let (xml, json) = dump(repro!("z14_name_line_opening_with_and"));
  assert_eq!(reading(&json), [
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

/// A note inside a link is an event of its own after it (2507.22485): the link's text its name, its ORCID its contact.
#[test]
fn a_note_inside_a_link_is_an_event_of_its_own() {
  let (xml, json) = dump(repro!("z14_note_inside_an_orcid_link"));
  assert_eq!(reading(&json), [
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
  let (xml, json) = dump(repro!("z14_odd_piece_in_a_marked_row"));
  assert_eq!(reading(&json), [
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
/// `classify::separators` splits it; a binding that declares it keeps what it prints (OmniBus, `perfect_kernel_batchz2::a_break_a_binding_declares_stands_beside_the_events`).
#[test]
fn alignauthor_is_a_column_of_the_rehearsal() {
  let (xml, json) = dump(repro!("z14_alignauthor_column_of_a_raw_class"));
  nothing_held(&json);
  assert_eq!(reading(&json), [
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

/// A contact a link in it prints, the constructor's ink empty: its text the link's address (2603.01171; the e-mail was lost from the reading).
#[test]
fn a_url_in_a_contact_call_is_its_text() {
  let (xml, json) = dump(repro!("z14_url_address_in_a_contact_call"));
  assert_eq!(reading(&json), [
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
  let (xml, json) = dump(repro!("z14r2_entry_addresses_cut_under_a_names_row"));
  assert_eq!(reading(&json), [
    "C Ann Able [†]",
    "  a Univ A",
    "  c email ann@a.org",
    "C Bob Baker [‡]",
    "  a Univ B",
    "  c email bob@b.org",
  ]);
  assert_eq!(json["emitted"], true, "{json}");
  assert_eq!(json["readback"]["difference"], Value::Null, "{json}");
  assert_eq!(json["readback"]["given_back"], Value::Null, "{json}");
  creators_are(&xml, &[
    &["Ann Able †", "affiliation=Univ A", "email=ann@a.org"],
    &["Bob Baker ‡", "affiliation=Univ B", "email=bob@b.org"],
  ]);
  for (author, _, expected) in [
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
      // (HG1: each entry its mark's name, as IEEEtran prints them; the e-mail line's addresses spell no one, so they
      // are the block's — the old engine's declined cut gave both names both lines, the expectation DEL1 stopped
      // comparing)
      "marks answered, addresses the block's",
      &[
        &["Ann Able †", "affiliation=Univ A"][..],
        &["Bob Baker ‡", "affiliation=Univ B"],
        &["email=x@a.org", "email=y@b.org"],
      ][..],
    ),
  ] {
    let (xml, _) = dump(&document("IEEEtran", author));
    creators_are(&xml, expected);
  }
}

/// A role heading opens a row of names only where each piece shows a person — a name's cue or its mark (hep-ph0311131): a heading over words is a line of the
/// block, as before Z14 (review revZ14's REDs: `Editors: Springer Nature` was a person).
#[test]
fn a_role_heading_over_words_opens_no_names() {
  let (xml, json) = dump(repro!("z14r2_role_heading_over_words"));
  assert_eq!(reading(&json), [
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
    let (_, json) = dump(&document("article", author));
    assert_eq!(reading(&json), expected, "{author}");
  }
}

/// RED (review revZ14): a mark past a marked list's leading run is flattened into its text, `Town, USA\\ *ann@x.org` reading `USA1ann@x.org` (2411.03172, 2510.06608,
/// 2602.11758): each mark should be an event, the institute both authors', each address its author's.
#[test]
#[ignore = "RED: a marked list's mark past its leading run is flattened into the text (events/mod.rs)"]
fn a_mark_past_a_marked_lists_leading_run_is_an_event_red() {
  let (_, json) = dump(repro!("z14r2_marked_list_mark_past_the_leading_run_red"));
  assert_eq!(reading(&json), [
    "C Ann Able [1]",
    "  a Univ X, Town, USA",
    "  c email ann@x.org",
    "C Bob Baker [2]",
    "  a Univ X, Town, USA",
    "  c email bob@x.org",
  ]);
}

/// Z14r3: an entry kept taken out because today's prints its addresses in an item the address reader cuts is given back whole where the plan cannot make it cut —
/// `e--mail:` no label the cut reads (astro-ph0007423's own `\address` of addresses) —, and the plan writes the rest: the paper had been declined, every numbered line
/// on both authors. An address today's holds as an author's contact stays taken out (2310.19837, `an_entrys_addresses_are_cut_where_its_line_is_made`).
#[test]
fn an_entry_whose_cut_fails_is_given_back() {
  let (log, xml, json) = run(repro!("z14r3_entry_given_back_where_its_cut_fails"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(json["emitted"], true, "{json}");
  // (T1, `=2`, `perfect_kernel_batcht1::expect`: each address of the `e--mail:` line is the e-mail of the author
  // it spells)
  creators_are(&xml, &[
    &[
      "Ann Able 1,2",
      "address=1 Univ A, Town A",
      "address=2 Univ B, Town B",
      "email=ann@a.org",
    ],
    &[
      "Bob Baker 1,3",
      "address=1 Univ A, Town A",
      "address=3 Univ C, Town C",
      "email=bob@b.org",
    ],
  ]);
}

/// Z14r3: a block's line keeping an address the reading left in it is made where the pass after the writer would leave that address in today's item as printed
/// (2203.17007's `$^{\star \star}$Ericsson Research, Sweden, E-mail: firstname.secondname@…`, `insert::entry_line_addresses`): no line was made, and the paper
/// declined, the lines merged on the first author. Where that pass would give it to its carriers as their e-mail, the line is not made (2310.19837, the third row of
/// `an_entrys_addresses_are_cut_where_its_line_is_made`).
#[test]
fn a_line_keeping_an_address_the_pass_leaves_is_made() {
  let (log, _, json) = run(repro!("z14r3_line_keeping_a_left_address_is_made"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(json["emitted"], true, "{json}");
}
