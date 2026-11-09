//! Red/green guards for batch 100c: two Link fixes the frontmatter observer read right and its writer declined
//! (census127 clusters 6 and 3; prep `agent_reports/2026-11-02_prep99d_link.md`).
//! - A line of notes only after a `\\` (`\thanks`, `\footnote`, `\IEEEcompsocitemizethanks` and spacing), its notes
//!   marked in a marked block, joins the line above, its notes the names' (OXIDIZED_DESIGN_DIVERGENCES #957; `author_block/preprocess.rs`
//!   `notes_line_joins_names`, `author_block/notes.rs` `notes_only_line`; Perl shares the defect, Base_Utility.pool.ltxml
//!   :702-712). Witnesses 2609.17106, 2411.05212, 2312.10807.
//! - A boxed author block's re-part writes an e-mail item, the reading's address in its row's fonts, and a
//!   `\resizebox`'s sizes print no name ink (OD #958; `observer/writer/{plan,repart,apply}.rs` `Made::Email`,
//!   `observer/events/mod.rs` `KEYED_CONSTRUCTORS`). Witnesses 2609.39183, 2507.07465, 2107.12423, 2603.10436,
//!   2608.20401.
//!
//! Each guard converts its repros under `tools/perfect_kernel/repros/sectioning-frontmatter/` in process and asserts
//! no Error and no Warning, no document `<pubnote>`, and every creator whole, in order. One guard per code path: the
//! kernel's marked block (article, IEEEtran's cautious reader with its `grade_only_line`, a `\\[len]` gap: the
//! three equal to the block written without the break), the break's own shapes (`\\*[len]` and a spacer `\\`, a line
//! ending at `\and`), the boundary (an unmarked block's unmarked notes, and notes after an `Emails:` line, stay a line), a
//! note no name requests (the document's); the re-part's e-mail item
//! (a brace list written out, a group's fonts in a `minipage`, a cell's in a `tabular`), its fail-safes for names joined
//! across cells and for a person in a row an item joins (2609.37429), and the `\resizebox` lexer entry.
use super::{
  convert_with,
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batcht1::red_at_two,
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

/// Converts `tex` in process, checks it has no Error, no Warning and no document `<pubnote>`, and returns its creators,
/// each whitespace run one space, from the first `<creator` to the last `</creator>`.
#[track_caller]
fn creators_of(tex: &str) -> String {
  let (log, xml) = convert_with(tex, None);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  assert!(!xml.contains("<pubnote"), "{xml}");
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  xml[start..end].to_string()
}

/// The marked block of a line of notes only after the names: each note its marks' authors', as the same block written
/// without the break reads (`notes_only_line_without_a_break_control`) — in article, in IEEEtran (whose binding reads
/// the block cautiously), and past a `\\[3.0ex]` (2312.10807). Witnesses 2411.05212, 2609.17106.
#[test]
fn a_notes_only_line_after_the_names_is_theirs() {
  let control = r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Ann Able and Cat Cole are with Univ A, Town.</contact> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks"><sup>*</sup>Work done at Lab C.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Bob Baker is with Univ B, City.</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Ann Able and Cat Cole are with Univ A, Town.</contact> </creator>"#;
  if red_at_two(r#"wraps the note's * mark in italic as ∗ instead of the printed *"#) {
    return;
  }
  assert_eq!(
    creators_of(repro!("notes_only_line_without_a_break_control")),
    control
  );
  for tex in [
    repro!("notes_only_line_after_the_names_is_theirs"),
    repro!("notes_only_line_after_ieee_names_is_theirs"),
    repro!("notes_only_line_after_a_gap_is_the_names"),
  ] {
    assert_eq!(creators_of(tex), control);
  }
}

/// The break's own shapes: a `\\*[2pt]` and a spacer `\\` before the notes (`\footnote`s parted by a tie, each a note
/// contact of its mark's author), and a line of notes in each `\and` group, which ends at the `\and`.
#[test]
fn a_notes_only_line_past_a_starred_break_a_spacer_or_in_an_and_group() {
  if red_at_two(r#"keeps the italic line mark 1 in the note contact's text"#) {
    return;
  }
  assert_eq!(
    creators_of(repro!("notes_only_line_after_a_starred_break_and_a_spacer")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Note: " role="note">Univ A, Town.</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Note: " role="note">Univ B, City.</contact> </creator>"#
  );
  assert_eq!(
    creators_of(repro!("notes_only_line_in_each_and_group")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town.</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B, City.</contact> </creator>"#
  );
}

/// The mechanism's boundary, where a line of notes stays a line: an unmarked block's unmarked notes are the document's —
/// no author's by position, batch 86d's reading (`perfect_kernel_batch86d::ieee_membership_kept_after_closing_stop`;
/// 2112.14209, 2409.11755, 2510.15773, 2605.22853); and marked notes after an `Emails: …` line, no names line, leave the
/// address list read as one, each address its author's (1911.03356).
#[test]
fn a_notes_only_line_stays_a_line_after_unmarked_names_or_an_emails_line() {
  for (tex, creators, note) in [
    (
      repro!("notes_only_line_after_unmarked_names"),
      r#"<creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator>"#,
      ">Both authors are with Univ A, Town.</pubnote>",
    ),
    (
      repro!("notes_only_line_after_an_emails_line_stays"),
      r#"<creator role="author"> <personname>Ann Able<sup>∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Emails: " role="email">ann@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup>∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <contact name="Emails: " role="email">bob@b.edu</contact> </creator>"#,
      "<sup>*</sup>Equal contribution.</pubnote>",
    ),
  ] {
    let (log, xml) = convert_with(tex, None);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
    if red_at_two(r#"the notes-only line is not kept as its own line under the names"#) {
      return;
    }
    assert!(xml.contains(creators), "{xml}");
    assert_eq!(xml.matches("<pubnote").count(), 1, "{xml}");
    assert!(xml.contains(note), "{xml}");
  }
}

/// A line of notes only that mixes notes the names' marks request with one no mark labels: the requested notes are the
/// names', the other stays the document's — one pubnote, not the last name's note (FIDELITY_CHARTER R1; review rev100c
/// S1: 2609.17106's funding note, 2312.10807's `$^{*}$Equal contribution`) — and a mark its text prints is no name's
/// (`$^{\dagger}$Corresponding author`, review probe q11). Distinct from the guards above: the line is joined in part, its
/// other notes taken out of the block (`notes_the_names_request`).
#[test]
fn a_note_no_name_requests_stays_the_documents() {
  let tex = repro!("notes_only_line_with_a_note_no_name_requests");
  let dagger = tex.replace("Ann Able$^{1}$", "Ann Able$^{1,*}$").replace(
    r"\thanks{This work was funded by Grant X.}",
    r"\thanks{$^{\dagger}$Corresponding author: Bob.}",
  );
  assert_ne!(dagger, tex);
  for (tex, ann, note) in [
    (
      tex,
      "<personname>Ann Able</personname>",
      r#"<pubnote role="thanks">This work was funded by Grant X.</pubnote>"#,
    ),
    (
      &dagger,
      "<personname>Ann Able<sup>∗</sup></personname>",
      "</sup>Corresponding author: Bob.</pubnote>",
    ),
  ] {
    let (log, xml) = convert_with(tex, None);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
      xml.contains(&format!(
        r#"<creator role="author"> {ann} <contact name="Affiliation: " role="affiliation">Univ A, Town.</contact> </creator> <creator before=" " role="author"> <personname>Bob Bell</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town.</contact> </creator>"#
      )),
      "{xml}"
    );
    assert_eq!(xml.matches("<pubnote").count(), 1, "{xml}");
    assert!(xml.contains(note), "{xml}");
    assert!(!xml.contains("<note "), "{xml}");
  }
}

/// A boxed block's e-mail row: each address the reading spells out of a brace list an e-mail contact of the name it
/// spells, in its row's fonts — a `\texttt{…}` in a `tabular` (2507.07465), a `{\scriptsize\ttfamily …}` group under a
/// `\small` in a `minipage` (2609.39183), a `\footnotesize` cell (2608.20401) — and no `tabular` left in a name.
#[test]
fn a_boxed_blocks_emails_are_their_names_contacts() {
  if red_at_two(
    r#"writes a before=" " separator the off reading leaves off a boxed block's creator"#,
  ) {
    return;
  }
  assert_eq!(
    creators_of(repro!("boxed_block_emails_written_out_of_a_brace_list")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Email: " role="email"><text font="typewriter">ann@a.edu</text></contact> </creator> <creator role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <contact name="Email: " role="email"><text font="typewriter">bob@b.edu</text></contact> </creator> <creator role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Email: " role="email"><text font="typewriter">cat@a.edu</text></contact> </creator>"#
  );
  assert_eq!(
    creators_of(repro!("boxed_block_email_in_a_grouped_font")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="70%">ann@a.edu</text></contact> </creator> <creator role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="70%">bob@b.edu</text></contact> </creator> <creator role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="70%">cat@a.edu</text></contact> </creator>"#
  );
  assert_eq!(
    creators_of(repro!("boxed_block_email_in_a_cell_font")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="80%">bob.baker@b.org</text></contact> </creator>"#
  );
}

/// The re-part's fail-safe: names the reading joined across an alignment's cells (`Ann Able & \mbox{\ \ } & Bob
/// Baker`, 1302.4168) are never written as one person, now that their e-mails could be made; today's creator stays (RED
/// repro: two creators are right).
#[test]
fn names_joined_across_cells_are_never_one_written_person() {
  let (log, xml) = convert_with(repro!("boxed_block_names_in_two_columns"), None);
  if red_at_two(r#"a false Error (Stray alignment "&") on names joined across cells"#) {
    return;
  }
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  assert!(
    !xml.contains("<personname>Ann Able Bob Baker</personname>"),
    "{xml}"
  );
  assert!(xml.contains("{ann, bob}@a.edu"), "{xml}");
}

/// The re-part's fail-safe for a person in a row an item joins: two people as two nested `tabular`s in one cell
/// (2609.37429's `\FormatAuthor`), which the reading takes for one name whose institution row runs on over the second
/// person's rows, are never written as one person holding the other in a contact; today's creator stays (RED repro:
/// two creators are right). Distinct from the guard above: no name is joined, an item is (`joined_rows`).
#[test]
fn a_person_in_a_row_an_item_joins_is_never_a_contact() {
  let (log, xml) = convert_with(repro!("boxed_block_two_people_in_one_cell"), None);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  assert_eq!(xml.matches("<creator ").count(), 1, "{xml}");
  if red_at_two(r#"a person in an item-joined row is read as a contact"#) {
    return;
  }
  assert!(!xml.contains("<contact"), "{xml}");
  for row in [
    ">Ann Able</td>",
    ">Bob Bell</td>",
    ">bob.bell@epfl.ch</text></td>",
  ] {
    assert!(xml.contains(row), "{xml}");
  }
}

/// A `\resizebox` round a boxed block: its sizes print no name ink, so the block is re-parted (2107.12423).
#[test]
fn a_resizebox_round_a_boxed_block_prints_no_size() {
  if red_at_two(
    r#"writes a before=" " separator the off reading leaves off a resizeboxed block's creator"#,
  ) {
    return;
  }
  assert_eq!(
    creators_of(repro!("boxed_block_resizebox_sizes_print_nothing")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#
  );
}
