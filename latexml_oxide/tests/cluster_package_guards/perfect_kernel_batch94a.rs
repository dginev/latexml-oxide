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
      let mut out = String::new();
      let mut rest = element.as_str();
      while let Some(at) = rest.find(" xml:id=\"") {
        out.push_str(&rest[..at]);
        let after = &rest[at + " xml:id=\"".len()..];
        rest = &after[after.find('"').expect("closed id") + 1..];
      }
      out.push_str(rest);
      out.split_whitespace().collect::<Vec<_>>().join(" ")
    })
    .collect()
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
