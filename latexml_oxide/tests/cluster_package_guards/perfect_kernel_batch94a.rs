//! Guards for batch 94a (frontmatter refactor stage 7, "name parting last": the writer's re-part,
//! OXIDIZED_DESIGN_DIVERGENCES #928; `docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`). Where the observer's reading parts
//! the names and today's old reader parted them otherwise, the writer writes the creators again from the reading — each
//! name from its span's ink, its items copied from today's or made from their spans — and keeps them only when they read
//! as the reading and hold every word today's creators held. Each guard converts its repro under the shadow switch in
//! process and with the binary, eager and `--streaming` (`perfect_kernel_batch92a::assert_reading`), asserts the
//! reading's tuples and the XML's creators whole, and no Error or Warning; and the default mode (the fleet's) writes the
//! same XML, notes included. Every repro compiles clean with pdflatex (TL 2025).

use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::set_fm_shadow_override;
use serde_json::Value;

use super::{
  perfect_kernel_batch84b::text,
  perfect_kernel_batch92a::assert_reading,
  perfect_kernel_batch93a::{default_mode_creators, owned},
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// A name's span clipped where the next event starts, less the group it opens (`name_tokens`): the names after `\\ \bf
/// \Large \textbf{…}` carry the cover of `\textbf`'s argument in their ink's span, so the first of them spanned the
/// rest (1909.05311, aaai20). Today's old reader held "Cid Cole, Dee Dunn" as one creator (an Error,
/// `merged_creators`); written, four creators, each its affiliation made from its `\ `-separated marked line — and the
/// Error, which reads the authors as written (`flag_merged_creators` after the writer), is gone.
#[test]
fn names_after_a_bold_line_are_reparted() {
  let tex = repro!("author_names_reparted_after_a_bold_line.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X"],
    &["Bob Baker", "affiliation=Univ Y"],
    &["Cid Cole", "affiliation=Univ X"],
    &["Dee Dunn", "affiliation=Univ Y"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request",
    "C Cid Cole [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Dee Dunn [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// A layout inside `\author{}` that today's old reader keeps as one creator's name: a minipage of `\quad`-parted names
/// and marked lines (2609.32870), and a `\makebox{\begin{tabular}…}` of comma-parted names, institution rows and a
/// `$^{\dagger}$` legend row (2609.33104). The word check reads a layout's cells and rows as parting the words beside
/// them (`dom::words_without_marks`: the name in one row and the institution in the next read glued otherwise); an
/// affiliation in a `{\small\normalfont …}` group keeps its switches; the legend is the dagger name's note (ruling 2,
/// `Made::Legend`), and so is the asterisk line in the minipage, "Equal advising." an equal share of the work as "Equal
/// contribution" is (`classify_thanks`; it was the starred name's affiliation: review rev94a S4). The names are the
/// runs of today's digested personname (`slice_name`): no `width`/`align` of the `\makebox` on them.
#[test]
fn names_in_a_layout_are_reparted() {
  let tex = repro!("author_names_reparted_in_a_minipage.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X"],
    &["Bob Baker", "affiliation=Univ Y"],
    &["Cid Cole", "affiliation=Univ X"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Bob Baker [2] | line:names sep:gap",
    "  a Univ Y | ruling1:request",
    "C Cid Cole [1,*] | line:names sep:gap",
    "  a Univ X | ruling1:request",
    "  n Equal advising. | ruling2:symbol-line",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Univ Y"],
      &["Cid Cole", "affiliation=Univ X", "note=∗ Equal advising."],
    ])
  );
  let tex = repro!("author_names_reparted_in_a_tabular_with_a_legend.tex");
  let reading = assert_reading(
    tex,
    Some(&[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Univ Y"],
      &["Cid Cole", "affiliation=Univ Y"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | ruling1:request",
      "C Bob Baker [2] | line:names sep:comma",
      "  a Univ Y | ruling1:request",
      "C Cid Cole [2,†] | line:names sep:comma",
      "  a Univ Y | ruling1:request ruling1:parted",
      "  n Corresponding author | ruling2:symbol-line ruling1:parted",
    ],
  );
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Univ Y"],
      &[
        "Cid Cole",
        "affiliation=Univ Y",
        "note=† Corresponding author"
      ],
    ])
  );
}

/// An entry of several pieces, which the reading sources at its first Name (`entry_span`): the writer reads on through
/// the events after it until their ink is the entry's text — the word "and" inside it (`Department of Physics and
/// Astronomy`, 1304.1574's `Computer Science and Engineering`) the entry's —, from a minipage of `$^1${\it …}` lines
/// today's old reader kept in the one creator's name with `Ann Able$^{\,1}$ and~ Bob Baker$^{\,2}$` (2307.03384); the
/// "and" between the names a separator the block printed, which no creator keeps.
#[test]
fn an_entry_of_several_pieces_is_written_whole() {
  let tex = repro!("author_names_reparted_entries_of_several_pieces.tex");
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=Department of Physics and Astronomy, Univ X, Japan",
    ],
    &[
      "Bob Baker",
      "affiliation=Department of Physics, Univ Y, Japan",
    ],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Department of Physics and Astronomy, Univ X, Japan | ruling1:request",
    "C Bob Baker [2] | line:names sep:gap",
    "  a Department of Physics, Univ Y, Japan | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// What today's name held besides the name moves to where the reading puts it — but a remark on the name stays the
/// name's: the line under it, `{\footnotesize for the Example collaboration}`, is the collaboration the name speaks
/// for (`is_name_remark`; review rev94a S6), one personname as today's and Perl's (1502.03246: 94a made it an
/// affiliation); and `\and` columns whose last is a line of marked entries over an e-mail line, each piece a creator
/// of today's (2308.10621): the legend is the starred names' note, the entry the line under it continues read on over
/// the break (`line:continued`), each item in print order.
#[test]
fn what_a_name_held_is_written_where_the_reading_puts_it() {
  let tex = repro!("author_names_reparted_a_line_under_the_name.tex");
  let xml: &[&[&str]] = &[&[
    "Ann Able for the Example collaboration",
    "affiliation=Univ X, Sometown, Country",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able for the Example collaboration [1] | line:names name:remark",
    "  a Univ X, Sometown, Country | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  let tex = repro!("author_names_reparted_columns_with_a_continued_line.tex");
  let reading = assert_reading(
    tex,
    Some(&[
      &["Ann Able", "affiliation=Univ X"],
      &[
        "Bob Baker",
        "affiliation=Univ X",
        "affiliation=Lab Y ann.able, bob.baker@lab.org",
      ],
      &["Cid Cole", "affiliation=Univ X"],
    ]),
    &[
      "C Ann Able [*,1] | line:names",
      "  a Univ X | ruling1:request",
      "  n equal contribution | ruling2:symbol-line",
      "C Bob Baker [*,1,2] | line:names sep:column",
      "  a Univ X | ruling1:request",
      "  a Lab Y ann.able, bob.baker@lab.org | ruling1:request line:continued",
      "  n equal contribution | ruling2:symbol-line",
      "C Cid Cole [1] | line:names sep:column",
      "  a Univ X | ruling1:request",
    ],
  );
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[
      &[
        "Ann Able",
        "affiliation=Univ X",
        "note=∗ equal contribution"
      ],
      &[
        "Bob Baker",
        "affiliation=Univ X",
        "affiliation=Lab Y ann.able, bob.baker@lab.org",
        "note=∗ equal contribution"
      ],
      &["Cid Cole", "affiliation=Univ X"],
    ])
  );
}

/// An `\affil{$^1$…}` inside `\author` (the paper's own `\newcommand{\affil}[1]{\\{\footnotesize #1}}`; 2405.08158,
/// 1602.05576: ar-1col's): one Contact event, which kept its mark's ink ("1Department of Physics, Univ X") and went to
/// the name beside it, so the re-part was taken back by its fail-safe and both lines stayed in the last name. Since 94b
/// (OD #937) the contact's leading Mark is its label and its span its content from the mark on: the names request
/// their lines (`ruling1:request`) and the re-part writes each, its leading mark out — a made affiliation carries no
/// entry mark, as today's copies (review rev94a S3: 94b wrote `<sup>1</sup>Department …`).
#[test]
fn an_affil_in_the_author_led_by_its_mark_is_reparted() {
  let tex = repro!("author_names_reparted_declined_mark_in_contact.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Department of Physics, Univ X"],
    &["Bob Baker", "affiliation=Department of Physics, Univ Y"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Department of Physics, Univ X | ruling1:request",
    "C Bob Baker [2] | line:names sep:word",
    "  a Department of Physics, Univ Y | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// Each `<ltx:personname>` of `xml`, whole: its ids out, its whitespace runs one space.
fn personnames(xml: &str) -> Vec<String> {
  xml
    .split("<personname")
    .skip(1)
    .map(|rest| {
      let end = rest.find("</personname>").expect("closed personname");
      let element = format!("<personname{}</personname>", &rest[..end]);
      without_ids(&element)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
    })
    .collect()
}

/// `xml` without its elements' `xml:id`s.
fn without_ids(xml: &str) -> String {
  let mut out = String::new();
  let mut rest = xml;
  while let Some(at) = rest.find(" xml:id=\"") {
    out.push_str(&rest[..at]);
    let after = &rest[at + " xml:id=\"".len()..];
    rest = &after[after.find('"').expect("closed id") + 1..];
  }
  out.push_str(rest);
  out
}

/// The default mode's XML of `tex`, no Error or Warning.
fn default_mode_xml(tex: &str) -> String {
  let (log, xml, ()) = convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    || set_fm_shadow_override(None),
    |_| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// The word "and" after an institution read in a row of names continues it in the reading (review rev94a B1): `Ann
/// Able, Department of Physics and Astronomy, Univ X` gives Ann Able "Department of Physics and Astronomy" and "Univ
/// X" — 94a made an author "Astronomy" of the word after the "and". The reading's institutions are the classify
/// helpers' (`classify:institution`), word evidence that licenses no item of today's creators (review rev94a r3,
/// `repart::structural`): the re-part declines and today's four creators stay, as 93a's — round 2 wrote them, the
/// accepted price of the rule.
#[test]
fn an_institution_continues_over_and() {
  let tex = repro!("author_names_reparted_institution_and_continues.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able"],
    &["Department of Physics"],
    &["Astronomy"],
    &["Univ X"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Department of Physics and Astronomy | line:names classify:institution",
    "  a Univ X | line:names classify:institution",
  ]);
  assert_eq!(reading.3, Value::from("declined: the creators' names"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// The re-part makes no item of words today's held inside one name (review rev94a B1): amsart's `\author{Ann Able,
/// Department of Physics and Astronomy}` is one creator as written, as Perl's — the classify helpers' institution is no
/// boundary the page shows, so the re-part declines (`repart::by_words`) and today's name stays (the standing record of
/// perfect_kernel_batch63's `John Smith, University of Toronto`).
#[test]
fn an_institution_inside_a_name_is_not_carved() {
  let tex = repro!("author_names_reparted_institution_in_a_name_declined.tex");
  let reading = assert_reading(
    tex,
    Some(&[&["Ann Able, Department of Physics and Astronomy"]]),
    &[
      "C Ann Able [] | line:names",
      "  a Department of Physics and Astronomy | line:names classify:institution",
    ],
  );
  assert_eq!(reading.3, Value::from("declined: the creators' names"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[&["Ann Able, Department of Physics and Astronomy"]])
  );
}

/// Inside an open bracket a comma or the word "and" is the bracket's (review rev94a B2, the lexer's `open`): `Cid Cole
/// (for the ATLAS and CMS Collaborations)` is one name, as today's old reader reads it — 94a made "CMS
/// Collaborations)" an author (0811.3142's `(on behalf of the H1 and ZEUS collaborations)`).
#[test]
fn a_bracket_holds_its_and() {
  let tex = repro!("author_names_reparted_brackets_hold_and.tex");
  let xml: &[&[&str]] = &[&["Ann Able"], &["Bob Baker"], &[
    "Cid Cole (for the ATLAS and CMS Collaborations)",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "C Bob Baker [] | line:names sep:comma",
    "C Cid Cole (for the ATLAS and CMS Collaborations) [] | line:names sep:word",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// A bracket after a name, its mark between, is a remark on the name (review rev94a B2, `is_name_remark`; the
/// collaboration a name speaks for is one too, 1502.03246): `Ann Able$^1$ (on behalf of the A and B teams)` one name,
/// whose mark requests Univ X — 94a wrote the authors "(on behalf of the A" and "B teams)".
#[test]
fn a_bracket_after_a_mark_is_the_names() {
  let tex = repro!("author_names_reparted_bracket_after_a_mark.tex");
  let xml: &[&[&str]] = &[&[
    "Ann Able (on behalf of the A and B teams)",
    "affiliation=Univ X",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able (on behalf of the A and B teams) [1] | line:names name:qualifier",
    "  a Univ X | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// Two marked institute groups on one line, one contact today, cut at their marks for the names requesting them, each
/// made from its own ink — no mark in a made affiliation, as today's copies (review rev94a B3, S3): 94a read each line
/// from its mark, whose formula's `$` the group closing after it left unbalanced, so the cut declined and the joined
/// contact stayed — Ann Able and Cid Cole held "Univ X 2 Univ Y", Bob Baker nothing (2308.11662). The mark `2` the cut
/// answered is no longer shown on Bob Baker's and Cid Cole's names (`hide_answered_marks`: today's showed it for no
/// entry of its own, `show_unanswered_marks`; pdflatex prints "Bob Baker2" over "2 Univ Y", the mark the entry's label
/// — review rev94a r2 S3, as Ann Able's `1` was always taken).
#[test]
fn two_marked_groups_on_a_line_are_cut() {
  let tex = repro!("author_names_two_marked_groups_cut.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X"],
    &["Bob Baker", "affiliation=Univ Y"],
    &["Cid Cole", "affiliation=Univ X", "affiliation=Univ Y"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request",
    "C Cid Cole [1,2] | line:names sep:comma mark:after-comma",
    "  a Univ X | ruling1:request",
    "  a Univ Y | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// A re-parted name is the run of today's digested personname its words are (review rev94a S1, S2, `slice_name`): its
/// smallcaps and its `\footnotemark[1]` kept (the note's id given back), its marks out — and no font of the document's
/// end, `\small` after `\maketitle`, on the names or the items made (94a: `fontsize="90%"` on each; 1607.00911,
/// 1909.05311, 2304.09381).
#[test]
fn reparted_names_keep_their_fonts_and_notes() {
  let tex = repro!("author_names_reparted_keep_fonts_and_notes.tex");
  let reading = assert_reading(
    tex,
    Some(&[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker * * footnote *", "affiliation=Univ Y"],
      &["Cid Cole * * footnote *", "affiliation=Univ X"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | ruling1:request",
      "C Bob Baker [2,*] | line:names sep:gap",
      "  a Univ Y | ruling1:request",
      "C Cid Cole [1,*] | line:names sep:gap",
      "  a Univ X | ruling1:request",
    ],
  );
  assert_eq!(reading.3, Value::from("written"));
  let xml = default_mode_xml(tex);
  let mark = "<note mark=\"*\" role=\"footnotemark\"><tags> <tag>*</tag> <tag role=\"refnum\">*</tag> <tag \
              role=\"typerefnum\">footnote *</tag> </tags></note>";
  assert_eq!(personnames(&xml), [
    "<personname><text font=\"smallcaps\">Ann Able</text></personname>".to_string(),
    format!("<personname>Bob Baker{mark}</personname>"),
    format!("<personname>Cid Cole{mark}</personname>"),
  ]);
  assert!(
    !xml.contains("fontsize=\"90%\"><text") && xml.matches("fontsize").count() == 1,
    "{xml}"
  );
  assert_eq!(xml.matches("xml:id=\"footnotex1\"").count(), 1, "{xml}");
  assert_eq!(xml.matches("xml:id=\"footnotex2\"").count(), 1, "{xml}");
}

/// The fail-safe: what the re-part makes is digested at the document's end, where the writer runs; an entry printing
/// through a macro redefined after `\maketitle` digests otherwise there ("Univ Q"), so what was built does not read as
/// the reading ("Univ X") and is taken back — today's one creator holding the minipage stays.
#[test]
fn a_repart_that_reads_otherwise_is_taken_back() {
  let tex = repro!("author_names_reparted_taken_back_state_changed.tex");
  let reading = assert_reading(tex, None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Bob Baker [2] | line:names sep:gap",
    "  a Univ Y | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("declined: the creators' names"));
  let xml = default_mode_xml(tex);
  assert_eq!(xml.matches("<creator ").count(), 1, "{xml}");
  assert!(xml.contains("Univ X") && !xml.contains("Univ Q"), "{xml}");
}

/// A name with the collaboration it speaks for after it is a name (review rev94a r2 B-1, `name_before_remark`): `Ann
/// Able and Bob Baker for the ATLAS Collaboration` is two authors, as today's — 94a r2 read "Collaboration" as an
/// institution's word and gave Bob Baker to Ann Able as her affiliation, the re-part made of today's whole creator.
#[test]
fn a_name_speaking_for_a_collaboration_is_a_name() {
  let tex = repro!("author_names_speaks_for_tail_is_a_name.tex");
  let xml: &[&[&str]] = &[&["Ann Able"], &["Bob Baker for the ATLAS Collaboration"]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "C Bob Baker for the ATLAS Collaboration [] | line:names sep:word",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// A full name before an institution word may be a person and a place, or one place (`Bob Baker University`, `Johns
/// Hopkins University`): an institution the classify helpers read from a creator's words is no structural evidence,
/// so no item is made of it (review rev94a r3, `repart::structural`; r2 B-1's word check `whole_creators` is gone):
/// the writer gives the creator back (`repart::concede`, `same`) and today's two creators stay — 94a r2 demoted Bob
/// Baker to Ann Able's affiliation.
#[test]
fn a_name_before_an_institution_word_stays_a_creator() {
  let tex = repro!("author_names_person_before_institution_word_kept.tex");
  let xml: &[&[&str]] = &[&["Ann Able"], &["Bob Baker University"]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Bob Baker University | line:names classify:institution",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// An institution read in a row of names continues over "and" or a comma only into a piece that may name no person
/// (review rev94a r2 B-1, `may_name_a_person`, the shadow reading's): `Ann Able, University of Toronto and Bob Baker
/// for the CMS Collaboration` reads Ann Able's institution and Bob Baker an author — 94a r2 gave Ann Able both pieces.
/// The institution is the classify helpers' (word evidence), so no item is made of today's creator "University of
/// Toronto" (review rev94a r3, `repart::structural`): the writer gives it back (`repart::concede`, `same`) and today's
/// three creators stay, as 93a's.
#[test]
fn an_institution_stops_at_a_person() {
  let tex = repro!("author_names_institution_stops_at_a_person.tex");
  let xml: &[&[&str]] = &[&["Ann Able"], &["University of Toronto"], &[
    "Bob Baker for the CMS Collaboration",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a University of Toronto | line:names classify:institution",
    "C Bob Baker for the CMS Collaboration [] | line:names sep:word",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// Invisible ink is no name's (review rev94a r2 B-2: the `\phantom` arm of the event lexer, `dom::is_phantom`):
/// `Ann Able\thanks{A} \phantom{a}and Bob Baker\thanks{B}` is two names parted by a space and "and", each its note —
/// 94a r2 wrote a creator `<phantom>a</phantom>and Bob Baker`, today's one creator of both (hep-th9806168) —; and a
/// `\phantom{agga}` after a name and its footnote is no author: the reading's one name pairs with today's, whose phantom
/// reads as a space (0909.3056, 1507.02405 — 94a r2 a second creator of the phantom alone).
#[test]
fn phantom_ink_is_no_name() {
  let tex = repro!("author_names_phantom_is_no_ink.tex");
  let reading = assert_reading(tex, Some(&[&["Ann Able"], &["Bob Baker"]]), &[
    "C Ann Able [] | line:names",
    "  n A | note:adjacent",
    "C Bob Baker [] | line:names sep:word",
    "  n B | note:adjacent",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[&["Ann Able", "note=A"], &["Bob Baker", "note=B"]])
  );
  let xml = default_mode_xml(tex);
  assert!(!xml.contains("ltx_phantom"), "{xml}");
  let tex = repro!("author_names_phantom_after_a_name_is_no_author.tex");
  let reading = assert_reading(tex, None, &[
    "C A. Able [] | line:names",
    "  n Dept X, Univ Y email: a@b.org fax: 39-0 | note:adjacent",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  let xml = default_mode_xml(tex);
  assert_eq!(xml.matches("<creator ").count(), 1, "{xml}");
}

/// A creator of today's becomes an item of another creator only on structural evidence — the reading's item answers a
/// mark, is a labelled line or row, or came from an API call (review rev94a r3, `repart::structural`): the classify
/// helpers' institution read from a creator's words licenses no item: the writer gives the creator back
/// (`repart::concede`), its reading today's (`same`), and today's creators stay, as 93a's. `Ann Able and Bob Baker for the Pierre Auger Observatory` (94a r3: Bob Baker Ann
/// Able's affiliation, "Observatory" an institution word; review r3 p11, p24-p26 its siblings) and `Ann Able, Bob
/// Baker, and the PE11 team of {\it Kepler} WG\#1` (94a r3: the team Bob Baker's affiliation; p33-p35), the team's
/// name in its italic as today's digested it.
#[test]
fn a_creator_becomes_an_item_only_on_structure() {
  let tex = repro!("author_names_creator_item_needs_structure.tex");
  let xml: &[&[&str]] = &[&["Ann Able"], &[
    "Bob Baker for the Pierre Auger Observatory",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Bob Baker for the Pierre Auger Observatory | line:names classify:institution",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  let tex = repro!("author_names_team_of_stays_a_creator.tex");
  let xml: &[&[&str]] = &[&["Ann Able"], &["Bob Baker"], &[
    "the PE11 team of Kepler WG#1",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "C Bob Baker [] | line:names sep:comma",
    "  a the PE11 team of Kepler WG#1 | line:names classify:institution",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  assert_eq!(personnames(&default_mode_xml(tex)), [
    "<personname>Ann Able</personname>",
    "<personname>Bob Baker</personname>",
    "<personname>the PE11 team of <text font=\"italic\">Kepler</text> WG#1</personname>",
  ]);
}

/// Where the shadow reading took one whole creator of today's for another's affiliation from its words alone, the
/// writer gives the creator back (`repart::concede`) and writes what the structure shows (review rev94a r3): `…, Bob
/// Baker$^1$, and the PE11 team of {\it Kepler} WG\#1\\ {$^*$} Affiliations …` keeps the team a creator of its own,
/// and the `$^*$` line, today's item on the team, goes to Ann Able, who shows its mark (1109.6488's García; the `1`
/// no line answers stays shown, as pdflatex prints "Ann Able*,1") — 94a r3 made the team Bob Baker's affiliation; a
/// decline alone would leave the line on the team.
#[test]
fn a_creator_read_by_its_words_is_given_back() {
  let tex = repro!("author_names_word_read_creator_conceded.tex");
  let reading = assert_reading(tex, None, &[
    "C Ann Able [*,1] | line:names",
    "  a Affiliations are given at the end of the paper | ruling2:symbol-line",
    "C Bob Baker [1] | line:names sep:comma",
    "  a the PE11 team of Kepler WG#1 | line:names classify:institution",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[
      &[
        "Ann Able 1",
        "affiliation=* Affiliations are given at the end of the paper"
      ],
      &["Bob Baker 1"],
      &["the PE11 team of Kepler WG#1"],
    ])
  );
}

/// A conceded creator keeps the ink it printed (review rev94a r4 B1, `repart::give_back_ink`): the shadow reading gave
/// the marks after `Kitt Peak Observatory` — a piece it took for Ann Able's institution by its words — to Ann Able, and
/// 94a r4 gave the observatory back bare, so the writer wrote every entry those marks answer on Ann Able. Given back
/// with its marks and the entries they answer, each creator holds its own: a moved entry (`$^{2}$`: Univ Y the
/// observatory's, as 93a's), a shared one copied (`$^{1}$`) and a symbol legend the observatory's note alone
/// (`$^{1,\dagger}$`, ruling 2: written) — and the same beside the re-part (today's merged `Cid Cole, Dee Dunn`,
/// 1909.05311's shape: four names written, the observatory given back with its entry). Distinct paths: the plan's
/// `same`, the plan's write, the re-part's write.
#[test]
fn a_conceded_creator_keeps_its_marks() {
  let tex = repro!("author_names_conceded_creator_keeps_its_marks.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "affiliation=Univ X"], &[
    "Kitt Peak Observatory",
    "affiliation=Univ Y",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1,2] | line:names",
    "  a Kitt Peak Observatory | line:names classify:institution",
    "  a Univ X | ruling1:request",
    "  a Univ Y | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  let tex = repro!("author_names_conceded_creator_shared_mark_and_legend.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "affiliation=Univ X"], &[
    "Kitt Peak Observatory",
    "affiliation=Univ X",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1,1,†] | line:names",
    "  a Kitt Peak Observatory | line:names classify:institution",
    "  a Univ X | ruling1:request",
    "  n Corresponding | ruling2:symbol-line",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[&["Ann Able", "affiliation=Univ X"], &[
      "Kitt Peak Observatory",
      "affiliation=Univ X",
      "note=† Corresponding",
    ]])
  );
  // (the legend note shows its printed mark: `apply::show_legend_mark`, review rev94a r5)
  assert_eq!(notes_with_marks(&default_mode_xml(tex)), [
    "mark=† † Corresponding"
  ]);
  let tex = repro!("author_names_reparted_conceded_creator_keeps_its_marks.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X"],
    &["Kitt Peak Observatory", "affiliation=Univ Y"],
    &["Cid Cole", "affiliation=Univ X"],
    &["Dee Dunn", "affiliation=Univ Y"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1,2] | line:names",
    "  a Kitt Peak Observatory | line:names classify:institution",
    "  a Univ X | ruling1:request",
    "  a Univ Y | ruling1:request",
    "C Cid Cole [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Dee Dunn [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// A made item is cut at its alignment's cells (review rev94a r4 B2, `plan::cell_tokens`): a two-column `tabular` in
/// `\author`, names over institutions — the reading's entry "Univ X Univ Y" spans the `&`, and made from its tokens
/// outside the alignment the `&` was a stray alignment tab, an Error that outlived the re-part's take-back (and with
/// one name, `Ann Able & `, 94a r4 wrote the joined affiliation). Neither is made, a taken-back re-part logs nothing of
/// what it digested, and today's one creator stays, 0 errors (93a's). An entry continued over the rows of one column
/// is made row by row, each row's tokens a group of its own, the row end the break it prints (nucl-th0306055's shape):
/// a cell's `\bf` stays its cell's — 94a r4 set the next row bold too.
#[test]
fn items_are_cut_at_an_alignments_cells() {
  for (tex, shadow, cells) in [
    (
      repro!("author_names_tabular_columns_no_item_across_cells.tex"),
      &[
        "C Ann Able Bob Baker [] | line:names name:layout-joined",
        "  a Univ X Univ Y | line:unmarked R12:row-above",
      ][..],
      &["Ann Able", "Bob Baker", "Univ X", "Univ Y"][..],
    ),
    (
      repro!("author_names_tabular_columns_one_name.tex"),
      &[
        "C Ann Able [] | line:names",
        "  a Univ X Univ Y | line:unmarked R12:row-above",
      ][..],
      &["Ann Able", "", "Univ X", "Univ Y"][..],
    ),
  ] {
    let reading = assert_reading(tex, None, shadow);
    assert_eq!(reading.0.len(), 1, "today's one creator: {:?}", reading.0);
    assert_eq!(reading.0[0].len(), 1, "no item: {:?}", reading.0);
    // (today's name the whole tabular, its cells in order: review rev94a r5 NIT)
    assert_eq!(tabular_cells(&default_mode_xml(tex)), cells);
    assert_eq!(default_mode_creators(tex), reading.0);
  }
  let tex = repro!("author_names_tabular_entry_over_rows_cut_per_row.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "affiliation=Univ X, Lab Z Dept Y"]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X, Lab Z Dept Y | ruling1:request line:continued",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  let xml = default_mode_xml(tex);
  let contact = without_ids(
    xml
      .split("<contact")
      .nth(1)
      .and_then(|rest| rest.split("</contact>").next())
      .expect("the affiliation"),
  );
  assert!(
    contact.ends_with(">Univ X, <text font=\"bold\">Lab Z<break/></text>Dept Y"),
    "{contact}"
  );
  // (a row end read as LaTeX reads it — `\\* [2pt]`, the star and the skip past the spaces before each — and the
  // spaces the next cell opens with none of its text: review rev94a r5 y01, t02, t12)
  let tex = repro!("author_names_tabular_row_end_star_and_skip.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "affiliation=Univ X, Dept Y"]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X Dept Y | ruling1:request line:continued",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(contacts(&default_mode_xml(tex)), [
    "Univ X,<break cssstyle=\"--ltx-break-space:2.0pt\"/>Dept Y"
  ]);
}

/// A row of a `tabular` in `\author` that may name a person is made an item only on structural evidence (review rev94a
/// r4): `Ann Able\\ Bob Baker` in a one-column tabular — 94a r1-r4 made "Bob Baker" Ann Able's affiliation by its place
/// under her name; today's one creator stays (93a's), as a line of today's creators does. The same decline holds for a
/// team's row and a name speaking for one (review r4 b03, b07, a23).
#[test]
fn a_person_in_a_tabular_row_is_no_item() {
  let tex = repro!("author_names_tabular_row_person_stays.tex");
  let reading = assert_reading(tex, None, &[
    "C Ann Able [] | line:names",
    "  a Bob Baker | line:unmarked R12:row-above",
  ]);
  assert_eq!(
    reading.0,
    [["Ann Able Bob Baker"]],
    "today's one creator, no item"
  );
  assert_eq!(tabular_cells(&default_mode_xml(tex)), [
    "Ann Able",
    "Bob Baker"
  ]);
  assert_eq!(reading.3, Value::from("declined: the creators' names"));
  assert_eq!(default_mode_creators(tex), reading.0);
}

/// The text of each `<td>` of `xml`, in document order (a name that is a whole tabular: today's one creator).
fn tabular_cells(xml: &str) -> Vec<String> {
  xml
    .split("<td")
    .skip(1)
    .map(|rest| {
      let start = rest.find('>').expect("td tag") + 1;
      // (an empty cell is `<td …/>`)
      if rest[..start].ends_with("/>") {
        return String::new();
      }
      let end = rest.find("</td>").expect("closed td");
      text(&rest[start..end])
    })
    .collect()
}

/// Each `ltx:note` of `xml` as `mark=<its mark attribute> <its text>`, in order (none: `mark=-`).
fn notes_with_marks(xml: &str) -> Vec<String> {
  xml
    .split("<note ")
    .skip(1)
    .map(|rest| {
      let open = rest.find('>').expect("note tag");
      let mark = rest[..open]
        .split("mark=\"")
        .nth(1)
        .and_then(|m| m.split('"').next())
        .unwrap_or("-");
      let close = rest.find("</note>").expect("closed note");
      format!("mark={mark} {}", text(&rest[open + 1..close]))
    })
    .collect()
}

/// Each `ltx:contact` of `xml`, its content whole (ids out).
fn contacts(xml: &str) -> Vec<String> {
  without_ids(xml)
    .split("<contact ")
    .skip(1)
    .map(|rest| {
      let open = rest.find('>').expect("contact tag") + 1;
      rest[open..rest.find("</contact>").expect("closed contact")].to_string()
    })
    .collect()
}

/// A legend answering several marks is each carrier's (review rev94a r5 g13, `repart::item_labels`): the conceded
/// observatory's `2` reads the `$^{1,2}$` line as its own too — its labels its legend's printed Mark, not the `1` the
/// reading keyed it by — so "Univ X" is copied to it and "Univ Y" moved, as pdflatex prints them (93a and r5: Univ Y
/// alone). Siblings on the same path: `$^{2,1}$`, `$^{1,*}$`, `\textsuperscript{1,2}`.
#[test]
fn a_legend_of_several_marks_is_each_carriers() {
  let tex = repro!("author_names_conceded_creator_legend_of_several_marks.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "affiliation=Univ X"], &[
    "Kitt Peak Observatory",
    "affiliation=Univ X",
    "affiliation=Univ Y",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1,2] | line:names",
    "  a Kitt Peak Observatory | line:names classify:institution",
    "  a Univ X | ruling1:request",
    "  a Univ Y | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// A legend written as a note shows its printed mark (review rev94a r5, `apply::show_legend_mark`): the note's `mark` is
/// the symbol its line prints, and the name shows it no more — 94a r1-r5 left the note without one, so the page set
/// the default dagger beside the name's own star ("Observatory*†"). The cut legend line beside a name (k01), and the
/// legend of a conceded creator's mark, its superscript kept in the name today's (f08).
#[test]
fn a_legend_note_shows_its_mark() {
  let tex = repro!("author_names_legend_note_shows_its_mark.tex");
  let reading = assert_reading(tex, None, &[
    "C Ann Able [*] | line:names",
    "  n Equal contribution | ruling2:symbol-line",
    "C Bob Baker [] | line:names sep:comma",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[&["Ann Able", "note=∗ Equal contribution"], &["Bob Baker"]])
  );
  let xml = default_mode_xml(tex);
  assert_eq!(personnames(&xml), [
    "<personname>Ann Able</personname>",
    "<personname>Bob Baker</personname>",
  ]);
  assert_eq!(notes_with_marks(&xml), ["mark=∗ ∗ Equal contribution"]);
  let tex = repro!("author_names_conceded_creator_legend_note_mark.tex");
  let reading = assert_reading(tex, None, &[
    "C Ann Able [*] | line:names",
    "  a Kitt Peak Observatory | line:names classify:institution",
    "  n Equal contribution | ruling2:symbol-line",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[&["Ann Able"], &[
      "Kitt Peak Observatory",
      "note=∗ Equal contribution"
    ]])
  );
  let xml = default_mode_xml(tex);
  assert_eq!(personnames(&xml), [
    "<personname>Ann Able</personname>",
    "<personname>Kitt Peak Observatory</personname>",
  ]);
  assert_eq!(notes_with_marks(&xml), ["mark=∗ ∗ Equal contribution"]);
}

/// A made item starts after its mark, its switches its own (review rev94a r5 u03, v02, v03; `plan::block_span_tokens`):
/// `$^1$\it Univ X` and `$^2$\small Univ Y` rows of a `tabular` in `\author` are made with their switch — 94a r5 began
/// them at their first Name, past the font Layout event that has no span, and lost it. Siblings on the same path:
/// `\bf`, `\sl`, `\em`, `\color{red}`, `\textit{…}`, `\small\it`.
#[test]
fn a_made_item_keeps_its_switch() {
  let tex = repro!("author_names_tabular_item_keeps_its_switch.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "affiliation=Univ X"], &[
    "Bob Baker",
    "affiliation=Univ Y",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(contacts(&default_mode_xml(tex)), [
    "<text font=\"italic\">Univ X</text>",
    "<text fontsize=\"90%\">Univ Y</text>",
  ]);
}

/// A cell's font reset is no names line (review rev94a r5 t06; `labels::later_line_is_names`): each cell of an
/// alignment opens in its font, so the row after a `\bf` row reads no font of its own and continues the institution
/// above it — 94a r5 wrote an author "City Z". Its rows read, the writer makes no item continued over a row that may
/// name a person, labelled or not (`repart::plan`, `Resolved::continued`): today's one creator stays (93a's).
#[test]
fn a_cells_font_reset_is_no_author() {
  let tex = repro!("author_names_tabular_font_reset_is_no_author.tex");
  let reading = assert_reading(tex, None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X Dept Y City Z | ruling1:request line:continued",
  ]);
  assert_eq!(reading.0.len(), 1, "today's one creator: {:?}", reading.0);
  assert_eq!(reading.0[0].len(), 1, "no item: {:?}", reading.0);
  assert_eq!(tabular_cells(&default_mode_xml(tex)), [
    "Ann Able 1",
    "1 Univ X,",
    "Dept Y",
    "City Z"
  ]);
  assert_eq!(default_mode_creators(tex), reading.0);
}

/// The writer cuts no author from a row of today's name on no structure (review rev94a r5 t06's mirror of the person
/// no item is made of; `repart::opens_a_row_under_no_names`): a row the reading reads as names after a labelled line
/// (`$^1$Univ X,\\ \bf City Z`) is an author only under a names row that ends with the name before it, or with a mark
/// or a note of its own — a line ending with an institution is none: today's one creator stays (94a r5 wrote "City Z"
/// an author). The supported side: `Ann Able, Bob Baker,\\ Cid Cole` is three authors (2407.03321's `\quad`-parted
/// rows).
#[test]
fn a_name_cut_from_a_row_needs_a_names_row() {
  let tex = repro!("author_names_tabular_row_after_an_institution_is_no_author.tex");
  let reading = assert_reading(tex, None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "C City Z [] | line:names",
  ]);
  assert_eq!(reading.0.len(), 1, "today's one creator: {:?}", reading.0);
  assert_eq!(reading.0[0].len(), 1, "no item: {:?}", reading.0);
  assert_eq!(tabular_cells(&default_mode_xml(tex)), [
    "Ann Able 1",
    "1 Univ X,",
    "City Z"
  ]);
  assert_eq!(default_mode_creators(tex), reading.0);
  let tex = repro!("author_names_tabular_names_row_after_a_comma.tex");
  let xml: &[&[&str]] = &[&["Ann Able"], &["Bob Baker"], &["Cid Cole"]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "C Bob Baker [] | line:names sep:comma",
    "C Cid Cole [] | line:names",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// An unlabelled affiliation a raw class prints in place after a run of names is every name of the run's (main
/// session 94a r6 item 8; the names line's `annotate=run` contact, `Resolver::record_run`): ci2018's `\affil` inside
/// `\author`, rerouted to `\lx@add@affiliation[annotate=run]`, after `JORGE RAMIREZ and EVGENY KRIVOSHEEV` (1805.12376)
/// — the reading gave it to the name beside it alone, and the re-part of today's merged names left the run's first
/// names with none. Today's API attaches it to the run (so the more owners are no ruling's change).
#[test]
fn an_affil_after_a_run_of_names_is_the_runs() {
  let tex = repro!("raw_class_affil_runs_inside_one_author.tex");
  let xml: &[&[&str]] = &[
    &["ANN ABLE", "affiliation=Univ A"],
    &["BOB BAKER", "affiliation=Univ A"],
    &["CID COLE", "affiliation=Univ B and Univ C"],
    &["DAN DOE", "affiliation=Univ B and Univ C"],
    &["EVE EGAN", "affiliation=Univ D"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C ANN ABLE [] | line:names",
    "  a Univ A | owner:annotate:run",
    "C BOB BAKER [] | line:names sep:word",
    "  a Univ A | owner:annotate:run",
    "C CID COLE [] | line:names",
    "  a Univ B and Univ C | owner:annotate:run",
    "C DAN DOE [] | line:names sep:word",
    "  a Univ B and Univ C | owner:annotate:run",
    "C EVE EGAN [] | line:names",
    "  a Univ D | owner:adjacent",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  // (the run ends at the block's contact of its role before it, labelled or not — review rev94a r6 B1,
  // `Resolver::ends_a_run`, today's API `marked_at`: a labelled entry waits for its label, its owner holding no role yet,
  // and r6 gave the later run's affiliation to the marked names before it too; a17, a14, a19)
  for (tex, xml, shadow) in [
    (
      repro!("raw_class_affil_run_ends_at_a_labelled_entry.tex"),
      &[
        &["ANN ABLE", "affiliation=Univ A"][..],
        &["BOB BAKER", "affiliation=Univ B"][..],
      ][..],
      &[
        "C ANN ABLE [1] | line:names",
        "  a Univ A | ruling1:request",
        "C BOB BAKER [] | line:names",
        "  a Univ B | owner:adjacent",
      ][..],
    ),
    (
      repro!("raw_class_affil_run_after_a_marked_name.tex"),
      &[
        &["ANN ABLE", "affiliation=Univ A"][..],
        &["BOB BAKER"][..],
        &["CID COLE", "affiliation=Univ B"][..],
      ][..],
      &[
        "C ANN ABLE [1] | line:names",
        "  a Univ A | ruling1:request",
        "C BOB BAKER [] | line:names sep:word",
        "C CID COLE [] | line:names",
        "  a Univ B | owner:adjacent",
      ][..],
    ),
    (
      repro!("raw_class_affil_run_ends_at_labelled_entries.tex"),
      &[
        &["ANN ABLE", "affiliation=Univ A"][..],
        &["BOB BAKER", "affiliation=Univ B"][..],
        &["CID COLE", "affiliation=Univ C"][..],
        &["DAN DOE", "affiliation=Univ C"][..],
      ][..],
      &[
        "C ANN ABLE [1] | line:names",
        "  a Univ A | ruling1:request",
        "C BOB BAKER [2] | line:names sep:word",
        "  a Univ B | ruling1:request",
        "C CID COLE [] | line:names",
        "  a Univ C | owner:annotate:run",
        "C DAN DOE [] | line:names sep:word",
        "  a Univ C | owner:annotate:run",
      ][..],
    ),
  ] {
    assert_reading(tex, Some(xml), shadow);
    assert_eq!(default_mode_creators(tex), owned(xml));
  }
}

/// An IEEEtran affiliation block's run (`annotate=new`, ieeetran_cls.rs) ends at the marked block before it (review
/// rev94a r6 B1, the same rule as a raw class's in-place `\affil`): `\IEEEauthorblockA{\IEEEauthorrefmark{1}Univ X}`
/// waits for its label, so the author above it held no affiliation yet and `\IEEEauthorblockA{Univ Y}` reached her too
/// (94b r2 and r6: Ann Able Univ Y and Univ X; i06); the run's names are those made after the marked block, so its
/// unmarked name is no name of the next block's either (i07: Bob Baker none). Both the observer's walk
/// (`Resolver::ends_a_run`) and today's API's (`frontmatter_api.rs` `marked_at`). Perl reads no mark here.
#[test]
fn an_ieee_affiliation_block_after_a_marked_block_is_the_next_names() {
  let tex = repro!("ieee_affil_block_after_a_marked_block.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able 1", "affiliation=Univ X"],
    &["Bob Baker", "affiliation=Univ Y"],
    &["Cid Cole", "affiliation=Univ Y"],
  ];
  assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | api:leading-mark",
    "C Bob Baker [] | line:names",
    "  a Univ Y | api:annotate:new",
    "C Cid Cole [] | line:names sep:word",
    "  a Univ Y | api:annotate:new",
  ]);
  assert_eq!(default_mode_creators(tex), owned(xml));
  let tex = repro!("ieee_affil_block_after_a_marked_block_of_two.tex");
  let xml: &[&[&str]] = &[&["Ann Able 1", "affiliation=Univ X"], &["Bob Baker"], &[
    "Cid Cole",
    "affiliation=Univ Y",
  ]];
  assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | api:leading-mark",
    "C Bob Baker [] | line:names sep:word",
    "C Cid Cole [] | line:names",
    "  a Univ Y | api:annotate:new",
  ]);
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// A legend of several symbols written as a note shows its printed mark (review rev94a r6 S1,
/// `apply::show_legend_mark`): `**` is no one glyph, so r6 left the note without a `mark` and the page set the default
/// dagger beside the name's own `**` ("Bob Baker** †"); its mark is now the note's leading superscript, which prints
/// the label's symbols, and the name shows it no more. Siblings on the path (`$^{\dagger\dagger}$`,
/// `$^{\ddagger\ddagger}$`, `\textsuperscript{**}`, `$^{1,**}$`) were probed alike.
#[test]
fn a_legend_of_several_symbols_shows_its_mark() {
  let tex = repro!("author_names_legend_of_several_symbols_shows_its_mark.tex");
  assert_reading(tex, None, &[
    "C Ann Able [*] | line:names",
    "  n Equal contribution | ruling2:symbol-line",
    "C Bob Baker [**] | line:names sep:comma",
    "  n Corresponding author | ruling2:symbol-line",
  ]);
  let xml = default_mode_xml(tex);
  assert_eq!(personnames(&xml), [
    "<personname>Ann Able</personname>",
    "<personname>Bob Baker</personname>",
  ]);
  assert_eq!(notes_with_marks(&xml), [
    "mark=∗ ∗ Equal contribution",
    "mark=∗∗ ∗∗ Corresponding author",
  ]);
}

/// The mark a legend note shows, taken from the name's superscript, leaves the superscript's font (review rev94a r6 N1,
/// `dom::take_sup_marks`, shared by `apply::show_legend_mark` and `hide_answered_marks`): r6 set the superscript's
/// content to the marks kept, which dropped the `\textbf` inside it (`<sup>2</sup>`; 93a `<sup><text
/// font="bold">2,*</text></sup>`).
#[test]
fn a_names_mark_taken_keeps_its_font() {
  let tex = repro!("author_names_legend_mark_taken_keeps_its_font.tex");
  assert_reading(tex, None, &[
    "C Ann Able [2,*] | line:names",
    "  n Corresponding author | api:leading-mark api:legend",
    "C Bob Baker [1] | line:names sep:word",
    "  a Univ X | api:leading-mark",
  ]);
  let xml = default_mode_xml(tex);
  assert_eq!(personnames(&xml), [
    "<personname>Ann Able<sup><text font=\"bold\">2</text></sup></personname>",
    "<personname>Bob Baker<sup><text font=\"italic\">1</text></sup></personname>",
  ]);
  assert_eq!(notes_with_marks(&xml), ["mark=* * Corresponding author"]);
}
