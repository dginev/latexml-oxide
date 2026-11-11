//! Red/green guards for batch 108a: authblk's `\author` reaches the kernel's author read on a separator it parts the block
//! at (`authblk_sty::has_separated_names`, `classify::has_author_block_separator`; OXIDIZED_DESIGN_DIVERGENCES #52(h)).
//! An argument with none stayed one name: every name and line in one `personname`, every note after it, each note's author
//! lost. Each guard converts its repros under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager
//! and `--streaming`, with 0 errors and 0 warnings, and asserts the frontmatter whole.
//!
//! Round 2 (review rev108a): an Error that goes away goes because the output became right, else it stays. A line under the
//! names that reads as a name is read only where the `\affil` next says so; a printed mark is shown where it prints or
//! counted; a piece with no name is no person.
//!
//! Round 3 (review rev108a2): a printed glyph and an order stay as written, and a line or address its author's block owns
//! stays there; whatever fails to read stays a counted Error.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batch89a::cli_conversion,
  perfect_kernel_batch99b::converted,
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

/// `A\thanks{…} \qquad B\thanks{…} \qquad C\thanks{…}` is three creators, each with its note (2610.06709). Distinct: the
/// note-to-author link, the defect's first symptom, by the kernel's own `\qquad` separator.
#[test]
fn qquad_parts_names_each_with_its_note() {
  assert_eq!(
    converted(repro!("authblk_thanks_qquad_parts_names"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note one.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Note two.</note> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id3">Note three.</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// The same with no notes at all. Distinct: nothing else in the argument reaches the kernel read (2608.05327).
#[test]
fn qquad_parts_names_without_notes() {
  assert_eq!(
    converted(repro!("authblk_names_qquad_parts_names"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// `\quad\quad` (2610.28573) and `\hspace{2.5em}` (2610.18435). Distinct: the second reaches the split only through
/// `normalize_hspace_separators`, which the first's empty piece between its separators must not undo.
#[test]
fn quad_quad_and_hspace_part_names_each_with_its_note() {
  assert_eq!(
    converted(repro!("authblk_thanks_quad_quad_parts_names"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note one.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Note two.</note> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id3">Note three.</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("authblk_thanks_hspace_parts_names"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note one.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Note two.</note> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id3">Note three.</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// Rows that must not change: `\and` two creators, two notes on one name one creator, a comma inside a note three
/// creators (2610.39888's accident, now by the `\qquad` too). Distinct: the gate's other branches and its negative.
#[test]
fn what_parted_or_stayed_whole_before_is_unchanged() {
  assert_eq!(
    converted(repro!("authblk_thanks_and_two_creators"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note one.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Note two.</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("authblk_two_thanks_one_name"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note one.</note> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Note two.</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("authblk_comma_in_note_parts_names"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Dept. X, Univ Y.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Note two.</note> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id3">Note three.</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A space between one person's notes (the shape of 2610.04108), inside a note, after the last note, before the name, or
/// before a protected `\footnotemark` (2609.40220) makes no second creator. Distinct: the negatives of the widened gate (`space_before_own_mark`, the groups `split_tokens`
/// leaves alone, the empty pieces).
/// FN1 (OD #1001; the user's ruling 2026-11-11): Eve Ede's mark is the creator's, right after `</personname>`, and
/// the space before it is trimmed with the name's end; pin updated (the note's move only).
#[test]
fn a_space_of_one_person_makes_no_second_creator() {
  assert_eq!(
    converted(repro!("authblk_spaces_of_one_person_part_nothing"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note one.</note> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Note two.</note> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id3">Dept. X Y.</note> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id4">Note three.</note> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Eve Ede</personname> <note mark="*" role="footnotemark" xml:id="footnotex1"><tags> <tag>*</tag> <tag role="refnum">*</tag> <tag role="typerefnum">footnote *</tag> </tags></note> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// `A\thanks{…}\\ Dept. of Physics` is the name with its note and its affiliation, not the name `ADept. of Physics`
/// (2609.17614); `\small` e-mail and institution lines are the name's contacts (2303.15964). Distinct: the `\\` row of the
/// gate.
#[test]
fn a_line_under_a_name_is_its_line() {
  assert_eq!(
    converted(repro!("authblk_line_under_name_is_its_line"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note one.</note> <contact name="Affiliation: " role="affiliation">Dept. of Physics</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("authblk_email_line_under_name"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id1">Univ X</text></contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="90%" xml:id="id2">bob.baker@x.org</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// The parted names are authblk's run: an unlabelled `\affil` is under each, and the next `\author`'s `\affil` is under that
/// one only (2610.23936). Distinct: `affil_run` counts what the list path queued.
#[test]
fn parted_names_share_the_affil_after_them() {
  assert_eq!(
    converted(repro!("authblk_parted_names_share_the_affil"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note one.</note> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ Y</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// Each parted name's mark asks its own affiliation; one creator holding both raised `merged_creators` (the one row of the
/// table with an Error before, 2610.07087). Distinct: the marks reach the list path's requests.
#[test]
fn parted_marked_names_ask_their_own_affiliations() {
  assert_eq!(
    converted(repro!("authblk_parted_marked_names_ask_theirs"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ Y</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A line under rows of names reads as a name where the block's institution is in the `\affil` next (2610.38214: "Stefano
/// Soatto" an affiliation of the other eleven, the merged Error gone). Distinct: `lines_of_names_before_affil`, the
/// rewritten route.
#[test]
fn a_lone_last_row_before_an_affil_is_a_name() {
  assert_eq!(
    converted(repro!("authblk_lone_last_row_name"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Eve Ede</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// The same line with no `\affil` next: nothing tells a name from "Waymo LLC" (2404.19531), so it is one creator and the
/// merged Error, the base's, not a name made an affiliation in silence. Distinct: the gate's refusal, the Error asserted.
#[test]
fn a_name_shaped_line_with_no_affil_keeps_the_error() {
  let (log, xml) = cli_conversion(repro!("authblk_name_shaped_line_keeps_the_error"), false);
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(
    log.matches("Error:frontmatter:merged_creators").count(),
    1,
    "{log}"
  );
  assert_eq!(creators(&xml), vec![vec![
    "Ann Able Bob Baker Cat Cole".to_string()
  ]]);
}

/// The `\affil` run is every creator of its `\author` call: names the kernel read welded and the reading parts all hold
/// it (2609.33075's first author lost it). Distinct: `Resolver::record_run`'s cap, `repart::plan`'s `same_run`.
/// T1 (`=2`, `perfect_kernel_batcht1::expect`): the same owners; the second creator keeps the space before it
/// as the third does.
#[test]
fn the_affil_run_is_every_creator_of_the_call() {
  assert_eq!(
    converted(repro!("authblk_first_row_keeps_run_affil"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole<sup xml:id="id3">†</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A mark glued in a list asks for each number and shows its symbol; one leading a line labels it by its number, the symbol
/// no text of the line (2606.23258). Distinct: `glued_marks`' list, `glued_line_marks_as_marks`.
#[test]
fn a_glued_mark_in_a_list_or_leading_a_line() {
  assert_eq!(
    converted(repro!("author_glued_mark_in_a_list"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> <contact name="Affiliation: " role="affiliation">Univ Y</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ Y</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A printed mark no entry answers and nothing can show is counted, never lost in silence. Distinct: the Warning.
#[test]
fn a_mark_that_stays_lost_is_counted() {
  let tex = repro!("author_unanswered_mark_lost_is_counted");
  let (log, xml) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(
    log
      .matches("Warning:frontmatter:unanswered_mark_lost")
      .count(),
    1,
    "{log}"
  );
  assert_eq!(creators(&xml), vec![vec!["Ann Able".to_string()], vec![
    "Bob Baker".to_string()
  ]]);
}

/// A star glued in math shows the `∗` it prints, in text the `*`, each mid-list in its place, the marks answered or not;
/// 2505.11121, 2609.14365. Distinct: `keepsup_in_math`, `GluedPart`, the comma of `show_unanswered_marks`.
/// Beside a second symbol too: `$^{2,1*\dagger}$` prints `∗†` (2404.16821, 2505.17017, 2609.38428, 2610.31762; rev108a3 B1).
#[test]
fn a_glued_star_beside_a_second_symbol_prints_as_written() {
  let (log, _) = cli_conversion(repro!("author_glued_star_beside_a_second_symbol"), false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
}

/// A list's order places what it spells no owner for only when the spelled addresses agree with it (FIDELITY_CHARTER §4):
/// `chang_liu@` second and Chang Liu third rule it out (2606.04646, 2605.26032, 2102.08716; rev108a3 B3).
#[test]
fn an_address_list_out_of_order_places_only_what_it_spells() {
  let (log, xml) = cli_conversion(
    repro!("email_list_spelled_out_of_place_has_no_order"),
    false,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Univ X", "email=ann@x.edu"],
    vec!["Bob Baker", "affiliation=Univ X"],
    vec!["Cat Cole", "affiliation=Univ X", "email=cat@x.edu"],
    vec!["Dan Doe", "affiliation=Univ X", "email=ddoe@x.edu"],
    vec!["Eve Elm", "affiliation=Univ X"],
    vec!["email=zz1@x.edu", "email=zz2@y.edu"],
  ]);
}

/// The mark-line route too (rev107g finding 4): `{bzz,dd,aql,cqr}@` spells `dd` out of place, so the rest stay on the block.
#[test]
fn a_mark_line_list_out_of_order_places_only_what_it_spells() {
  let (log, _) = cli_conversion(
    repro!("email_list_mark_line_spelled_out_of_place_no_order"),
    false,
  );
  assert_eq!(error_count(&log), 0, "{log}");
}

/// A row of addresses reads each address clean: no separator in a contact, a comma before the space none either
/// (2505.22665), ties around the space or a comma ending the row neither (2610.06195, 2505.05785). Distinct: the join's
/// `\quad`, `without_row_spacing`.
#[test]
fn a_row_of_addresses_reads_each_clean() {
  assert_eq!(
    converted(repro!("author_address_row_ties_and_end_comma"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email"><text font="typewriter" xml:id="id1">ann@x.org</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Email: " role="email"><text font="typewriter" xml:id="id2">bob@y.org</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("author_address_row_comma_before_the_space"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="e-mail: " role="email">ann@x.org</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="e-mail: " role="email">bob@y.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A place right under an institution of the whole row goes with it (2609.30627, 2609.03842). Distinct: the row's carriers
/// through `row_line_calls`, `reads_as_place`'s connectives.
#[test]
fn a_place_under_a_row_institution_stays_with_it() {
  assert_eq!(
    converted(
      repro!("author_place_under_a_row_institution_stays_with_it"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Dept of Statistics, Univ X</contact> <contact name="Affiliation: " role="affiliation">Seoul, Republic of Korea</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id1">able@x.edu</text></contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="∗" role="thanks" xml:id="id2"><sup xml:id="id2.1"><text font="italic" xml:id="id2.1.1">∗</text></sup>Corresponding author</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Dept of Statistics, Univ X</contact> <contact name="Affiliation: " role="affiliation">Seoul, Republic of Korea</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id3">bbaker@y.edu</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A macro printing a link alone among the pieces is no person: one name and the counted merge (2601.07891's `\githubrepo`),
/// the Error falling only when the output is right. Distinct: `holds_a_macro_printing_a_link`.
#[test]
fn a_macro_printing_a_link_keeps_the_error() {
  let (log, xml) = cli_conversion(
    repro!("authblk_macro_printing_a_link_keeps_the_error"),
    false,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(
    log.matches("Error:frontmatter:merged_creators").count(),
    1,
    "{log}"
  );
  assert_eq!(creators(&xml), vec![vec![
    "Ann Able * Bob Baker X/KV X/press".to_string()
  ]]);
}

/// A raw class's store under authblk is no `\affil`: no `\AB@thenote` prints, so no author requests entry 1 too
/// (2610.36456's Feng Xie). Distinct: `thenote_affil` in the observer's reading.
#[test]
fn a_class_store_makes_no_printed_note() {
  let (log, xml) = cli_conversion(repro!("authblk_class_store_makes_no_printed_note"), false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), vec![
    vec!["Ann Able".to_string(), "affiliation=Univ A".to_string()],
    vec!["Bob Baker".to_string(), "affiliation=Univ B".to_string()],
  ]);
}

/// The lost mark's Warning reads the number and the ORCID iD apart (2205.05709's `10000-0001-…`). Distinct: the message.
#[test]
fn a_lost_marks_warning_parts_the_orcid() {
  let (..) = cli_conversion(repro!("author_lost_mark_message_parts_the_orcid"), false);
}
