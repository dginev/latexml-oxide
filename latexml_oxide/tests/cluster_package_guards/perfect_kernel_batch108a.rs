//! Red/green guards for batch 108a: authblk's `\author` reaches the kernel's author read on a separator it parts the block
//! at (`authblk_sty::has_separated_names`, `author_block::has_author_block_separator`; OXIDIZED_DESIGN_DIVERGENCES #52(h)).
//! An argument with none stayed one name: every name and line in one `personname`, every note after it, each note's author
//! lost. Each guard converts its repros under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager
//! and `--streaming`, with 0 errors and 0 warnings, and asserts the frontmatter whole.
//!
//! Round 2 (review rev108a): an Error that goes away goes because the output became right, else it stays. A line under the
//! names that reads as a name is read only where the `\affil` next says so; a printed mark is shown where it prints or
//! counted; a piece with no name is no person.
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

/// `\hfill`, a run of ties and a spacing-only math group (2203.00958, 2410.10922). Distinct: the normalizer's other
/// separators, and `$\qquad$` outside `split_tokens`'s reach until it is unwrapped.
#[test]
fn other_spaces_the_kernel_reads_part_names() {
  assert_eq!(
    converted(repro!("authblk_other_spaces_part_names"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note one.</note> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Note two.</note> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Eve Ede</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Fay Fox</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
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
#[test]
fn a_space_of_one_person_makes_no_second_creator() {
  assert_eq!(
    converted(repro!("authblk_spaces_of_one_person_part_nothing"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note one.</note> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Note two.</note> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id3">Dept. X Y.</note> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id4">Note three.</note> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Eve Ede <note mark="*" role="footnotemark" xml:id="footnotex1"><tags> <tag>*</tag> <tag role="refnum">*</tag> <tag role="typerefnum">footnote *</tag> </tags></note></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
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

/// A font command around the whole of the names is transparent, as for the "and" word (2609.20129's `\bf` family, 2404.12241's
/// `\textbf` names). Distinct: `whole_wrapper_content`.
#[test]
fn a_wrapper_around_the_names_is_transparent() {
  assert_eq!(
    converted(repro!("authblk_wrapper_over_parted_names"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
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
#[test]
fn the_affil_run_is_every_creator_of_the_call() {
  assert_eq!(
    converted(repro!("authblk_first_row_keeps_run_affil"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator role="author"> <personname>Bob Baker<sup xml:id="id2">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole<sup xml:id="id3">†</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A number glued after a symbol asks for its entry and shows, and one before a symbol shows before it: 2505.22662's `*1`
/// was lost, 2610.07087's `1*` read `*1`. Distinct: the symbol-first split, the written order at `show_unanswered_marks`.
#[test]
fn a_glued_mark_shows_as_it_prints() {
  assert_eq!(
    converted(repro!("authblk_symbol_first_glued_mark_kept"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup><sup xml:id="id2">1</sup></personname> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3">2</sup></personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("authblk_number_first_glued_mark_in_order"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup><sup xml:id="id2">*</sup></personname> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3">2</sup></personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A mark glued in a list asks for each number and shows its symbol; one leading a line labels it by its number, the symbol
/// no text of the line (2606.23258). Distinct: `glued_marks`' list, `glued_line_marks_as_marks`.
#[test]
fn a_glued_mark_in_a_list_or_leading_a_line() {
  assert_eq!(
    converted(repro!("author_glued_mark_in_a_list"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> <contact name="Affiliation: " role="affiliation">Univ Y</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ Y</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("author_glued_mark_labels_its_line"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
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

/// A piece of addresses alone is no person: under the names, the row's list (2609.02424), beside a name in a wrapper, a
/// link or under a label. Distinct: `address_only_line` in the unmarked read; the row's join.
#[test]
fn a_piece_of_addresses_is_no_person() {
  assert_eq!(
    converted(repro!("authblk_address_piece_no_person"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann@x.org</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Email: " role="email">bob@y.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("authblk_address_beside_name_no_person"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann@x.org</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Email: " role="email">(bob@y.org)</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Email: " role="email"><ref class="ltx_href" href="mailto:cat@z.org">cat@z.org</ref></contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="E-mail: " role="email">dan@w.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// An address line never names the block: the link over it does, printing a bare address (2609.25082, "Sherpa.ai" the URL,
/// the address the name). Distinct: `link_prints_a_bare_address`.
#[test]
fn an_address_line_never_names_the_block() {
  assert_eq!(
    converted(repro!("authblk_address_line_names_no_block"), 0),
    r##"<title>T</title> <creator role="author"> <personname><ref class="ltx_href" fontsize="173%" href="https://x.ai/">X.ai</ref></personname> <contact name="Email: " role="email">research@x.ai</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A piece that prints no name is the name's before it: no second, empty creator. Distinct: `names_no_one`.
#[test]
fn a_piece_with_no_name_is_no_creator() {
  assert_eq!(
    converted(repro!("authblk_orcid_after_space_no_creator"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<ref class="ltx_orcid" href="https://orcid.org/0000-0000-0000-0000" title="ORCID 0000-0000-0000-0000"><svg:svg class="ltx_orcidlogo" height="1em" version="1.1" viewBox="0 0 72 72" width="1em"> <svg:path d="M72,36 C72,55.884375 55.884375,72 36,72 C16.115625,72 0,55.884375 0,36 C0,16.115625 16.115625,0 36,0 C55.884375,0 72,16.115625 72,36 Z" fill="#A6CE39"/> <svg:g fill="#FFFFFF" transform="translate(18.868966, 12.910345)"> <svg:polygon points="5.03734929 39.1250878 0.695429861 39.1250878 0.695429861 9.14431787 5.03734929 9.14431787 5.03734929 22.6930505 5.03734929 39.1250878"/> <svg:path d="M11.409257,9.14431787 L23.1380784,9.14431787 C34.303014,9.14431787 39.2088191,17.0664074 39.2088191,24.1486995 C39.2088191,31.846843 33.1470485,39.1530811 23.1944669,39.1530811 L11.409257,39.1530811 L11.409257,9.14431787 Z M15.7511765,35.2620194 L22.6587756,35.2620194 C32.49858,35.2620194 34.7541226,27.8438084 34.7541226,24.1486995 C34.7541226,18.1301509 30.8915059,13.0353795 22.4332213,13.0353795 L15.7511765,13.0353795 L15.7511765,35.2620194 Z"/> <svg:path d="M5.71401206,2.90182329 C5.71401206,4.441452 4.44526937,5.72914146 2.86638958,5.72914146 C1.28750978,5.72914146 0.0187670918,4.441452 0.0187670918,2.90182329 C0.0187670918,1.33420133 1.28750978,0.0745051096 2.86638958,0.0745051096 C4.44526937,0.0745051096 5.71401206,1.36219458 5.71401206,2.90182329 Z"/> </svg:g> </svg:svg></ref></personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// An empty mark is no mark, and the unmarked piece before the marked ones is the block's (user ruling R12), not on
/// `Baker$^2$` (2505.14999); a piece a mark leads stays the marked list's (2505.00989). Distinct:
/// `unmarked_before_carried_marks` and its negative.
#[test]
fn an_empty_marked_piece_is_the_blocks() {
  assert_eq!(
    converted(repro!("authblk_empty_mark_piece_is_the_blocks"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"> Univ Y</contact> </creator> <creator role="author"> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  // (a first piece a mark leads is the list's, its legend and entries each by their marks: 2505.00989)
  assert_eq!(
    converted(repro!("authblk_marked_first_piece_stays_the_lists"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup>Univ X</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2"><text font="italic" xml:id="id2.1">2</text></sup>Univ Y</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="∗" role="thanks" xml:id="id3"><sup xml:id="id3.1"><text font="italic" xml:id="id3.1.1">∗</text></sup>Corresponding author</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}
