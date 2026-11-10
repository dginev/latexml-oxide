//! Red/green guards for batch 100c: two Link fixes the frontmatter observer read right and its writer declined
//! (census127 clusters 6 and 3; prep `agent_reports/2026-11-02_prep99d_link.md`).
//! - A line of notes only after a `\\` (`\thanks`, `\footnote`, `\IEEEcompsocitemizethanks` and spacing), its notes
//!   marked in a marked block, joins the line above, its notes the names' (OXIDIZED_DESIGN_DIVERGENCES #957; `defs/author_macros/ijcai.rs`
//!   `notes_line_joins_names`, `frontmatter/classify/old_reader/notes.rs` `notes_only_line`; Perl shares the defect, Base_Utility.pool.ltxml
//!   :702-712). Witnesses 2609.17106, 2411.05212, 2312.10807.
//! - A boxed author block's re-part writes an e-mail item, the reading's address in its row's fonts, and a
//!   `\resizebox`'s sizes print no name ink (OD #958; `observer/emitter/{plan,repart}.rs` and the S2b writer's `apply.rs` (deleted in DEL4) `Made::Email`,
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

/// The mechanism's boundary, where a line of notes stays a line: an unmarked block's unmarked notes are the document's —
/// no author's by position, batch 86d's reading (`perfect_kernel_batch86d::ieee_membership_kept_after_closing_stop`;
/// 2112.14209, 2409.11755, 2510.15773, 2605.22853); and marked notes after an `Emails: …` line, no names line, leave the
/// address list read as one, each address its author's (1911.03356).
#[test]
fn a_notes_only_line_stays_a_line_after_unmarked_names_or_an_emails_line() {
  for (tex, ..) in [
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
    let (log, _) = convert_with(tex, None);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
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
}
