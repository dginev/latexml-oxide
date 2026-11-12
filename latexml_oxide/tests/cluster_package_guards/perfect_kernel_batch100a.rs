//! Red/green guards for batch 100a: "Deceased" is an annotation on the author (FIDELITY_CHARTER §4, user ruling
//! 2026-11-02), a contact line led by a control-sequence footnote symbol no name requests stays the class's text
//! (review rev99b2 F2), and 99b's routed lines holding several requested entries go to batch 99a's Cut (SYNC row "99b"
//! (e); OXIDIZED_DESIGN_DIVERGENCES #955). Code paths: the shared kind reader (`classify_thanks`'s `deceased`) read by
//! the kernel's line route (queue_digest.rs `legend_note_kind`) and by the frontmatter observer (`is_legend_text`); the
//! doubt readers of a routed line's leading mark (frontmatter_stores.rs `leads_with_unrequested_mark` on the run-in and
//! in-place routes, `displaced_in_doubt` on the displaced route: `is_a_mark`, `requested_as_printed`); and
//! `contact_line_call`'s handover to the API's Cut (`holds_requested_marked_entries`). Round 2 (review rev100a): the one
//! key both a name's request and a line's label go through (queue_digest.rs `clean_frontmatter_labels`, `one_asterisk`:
//! `*`, `∗` and `\ast` alike), the doubt test's one mark reader (`prints_a_mark`, the printed text) and "Deceased" naming
//! an institution an affiliation (`opens_with_deceased`, `reads_as_affiliation`). Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, and asserts the
//! frontmatter it makes whole.
use super::perfect_kernel_batch99b::converted;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A legend line `$^\dagger$Deceased` under the name carrying its symbol is that name's note of the `deceased` kind
/// (`ltx_thanks_deceased`), never its affiliation: on a raw class's run-in route (`legend_note_kind`; an address-change
/// legend beside it stays its carrier's note), in the kernel's `\author` block read by the frontmatter observer
/// (`is_legend_text`), and in a binding's affiliation entry the observer reads as a legend (resolver/api.rs, the
/// `api:leading-mark` legend: REVTeX's `$^\star$Deceased`). Review rev99b2 probe q4e.
#[test]
fn a_deceased_legend_is_a_note_of_its_carriers() {
  assert_eq!(
    converted(repro!("run_in_deceased_legend_is_a_note_of_its_carrier"), 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,†</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_deceased" mark="†" role="thanks" xml:id="id3"><sup xml:id="id3.1">†</sup>Deceased</note> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4"><text font="italic" xml:id="id4.1">2,∗</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id5">2</sup>Univ B, City</contact> <note class="ltx_note_frontmatter ltx_thanks_address" mark="*" role="thanks" xml:id="id6"><sup xml:id="id6.1">∗</sup>Present address: Univ X, Village</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("author_block_deceased_legend_is_a_note_of_its_carrier"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_deceased" mark="†" role="thanks" xml:id="id1"><sup xml:id="id1.1">†</sup>Deceased</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A run-in or in-place line led by a control-sequence footnote symbol no name requests (`\affil{$^\dagger$Deceased}`
/// under `Ann Able$^{1}$, Bob Baker$^{2}$`) is in doubt: the class's text where it prints, no author's contact (it was
/// every author's of the run: review rev99b2 F2, probes q4c, q4h). A symbol spelled otherwise on its carrier
/// (`\textsuperscript{\dag}` under `$^{1,\dagger}$`) is requested by its printed text and stays the carrier's note.
#[test]
fn a_run_in_or_in_place_line_led_by_an_unrequested_symbol_stays_the_classs() {
  assert_eq!(
    converted(
      repro!("run_in_line_led_by_an_unrequested_symbol_stays_the_classs"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3"><text font="italic" xml:id="id3.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4">2</sup>Univ B, City</contact> </creator> <para class="ltx_align_left" xml:id="p1"> <p xml:id="p1.1"><sup xml:id="p1.1.1"><text fontsize="90%" xml:id="p1.1.1.1">†</text></sup><text fontsize="90%" xml:id="p1.1.2">Deceased</text></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("in_place_line_led_by_an_unrequested_symbol_stays_the_classs"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3"><text font="italic" xml:id="id3.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4">2</sup>Univ B, City</contact> </creator> <para class="ltx_align_left" xml:id="p1"> <p xml:id="p1.1"><sup xml:id="p1.1.1"><text fontsize="90%" xml:id="p1.1.1.1">†</text></sup><text fontsize="90%" xml:id="p1.1.2">Deceased</text></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("run_in_symbol_line_spelled_otherwise_is_its_carriers_note"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,†</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_deceased" mark="†" role="thanks" xml:id="id3"><sup xml:id="id3.1">†</sup>Deceased</note> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id5">2</sup>Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A displaced store whose line leads with a control-sequence footnote symbol no name requests is in doubt, the whole
/// call the class's text where its title page prints it (`displaced_in_doubt`; review rev99b2 F2, probe q4d).
#[test]
fn a_displaced_line_led_by_an_unrequested_symbol_stays_the_classs() {
  assert_eq!(
    converted(
      repro!("displaced_line_led_by_an_unrequested_symbol_stays_the_classs"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">2</sup></personname> </creator> <para xml:id="p1"> <p xml:id="p1.1"><sup xml:id="p1.1.1"><text fontsize="90%" xml:id="p1.1.1.1">1</text></sup><text font="italic" fontsize="90%" xml:id="p1.1.2">Univ A, Town<break/><sup xml:id="p1.1.2.1">2</sup>Univ B, City<break/><sup xml:id="p1.1.2.2">†</sup>Deceased</text></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
  );
}

/// A sole author's run-in `\affil` lines led by the marks the name requests, then an in-place `\email`: the
/// affiliations come before the e-mail, as the PDF prints them (batch 99a's `relocate::at_queue_position`; the RED repro
/// of batch 99b, FIDELITY_CHARTER R8). Witnesses 2412.06969, 2503.13373, 2512.20289, 2603.07344, 2607.12158, 2609.01336,
/// 2609.32125, 2608.28486.
#[test]
fn run_in_lines_keep_their_print_order() {
  assert_eq!(
    converted(repro!("run_in_lines_a_run_places_keep_their_order"), 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id3">2</sup>Univ B, City</contact> <contact name="Email: " role="email">ann@univ-a.edu</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// An asterisk keys one way, written `*`, `\ast` or `\textasteriskcentered`, in text or math, printed in any form ("all
/// asterisks are the same key, they are visually identical to readers": user ruling 2026-11-02, FIDELITY_CHARTER §4;
/// queue_digest.rs `clean_frontmatter_labels`, `one_asterisk`), on both sides of the match: the name's request and the
/// line's label. So a legend `$^\ast$…` under
/// `Ann Able$^{1,*}$`, and `$^*$…` under `$^{1,\ast}$`, is its carrier's — on the run-in route, the displaced route
/// (`displaced_in_doubt`: keyed apart the whole store was in doubt and both authors lost every affiliation) and the
/// in-place route (a correspondence note) — the note's mark the glyph its line prints (`line_mark`); a text-mode
/// `\textsuperscript{\textasteriskcentered}` legend is read by the symbol it prints (`legend_note_kind`). Keyed apart, no
/// name seemed to request the line, which stayed the class's text in the body (review rev100a B1, probes q/p2, p2b, p2c,
/// p2d).
#[test]
fn an_asterisk_keys_alike_written_or_printed() {
  assert_eq!(
    converted(
      repro!("run_in_asterisk_legend_under_a_math_asterisk_mark_is_its_carriers_note"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,∗</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_deceased" mark="*" role="thanks" xml:id="id3"><sup xml:id="id3.1">∗</sup>Deceased</note> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id5">2</sup>Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("run_in_math_asterisk_legend_under_an_asterisk_mark_is_its_carriers_note"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,∗</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_deceased" mark="∗" role="thanks" xml:id="id3"><sup xml:id="id3.1">∗</sup>Deceased</note> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id5">2</sup>Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("displaced_math_asterisk_line_under_an_asterisk_mark_is_its_carriers"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id1">1</sup>Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_deceased" mark="∗" role="thanks" xml:id="id2"><sup xml:id="id2.1">∗</sup>Deceased</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id3">2</sup>Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("in_place_math_asterisk_legend_under_an_asterisk_mark_is_its_carriers_note"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,∗</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="∗" role="thanks" xml:id="id3"><sup xml:id="id3.1">∗</sup>Corresponding author</note> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id5">2</sup>Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("run_in_text_asterisk_legend_under_an_asterisk_mark_is_its_carriers_note"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,∗</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_deceased" mark="*" role="thanks" xml:id="id3"><sup xml:id="id3.1">*</sup>Deceased</note> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id5">2</sup>Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A line led by a symbol a command prints that no name requests — a suit (affiliation labels as digits are,
/// 2601.05633) or `\textsuperscript{\textdagger}` — is in doubt and stays the class's text: the doubt test reads a mark
/// by its printed text (`is_a_mark` = `prints_a_mark`), not a list of footnote symbols, under which `\bigstar`,
/// `\natural`, `\flat`, the suits and `\textdagger` were no mark and each line went to every author of the run (review
/// rev100a S1, probes q/su1, q/tx1).
#[test]
fn a_line_led_by_a_printed_symbol_no_name_requests_stays_the_classs() {
  assert_eq!(
    converted(
      repro!("run_in_lines_led_by_unrequested_printed_symbols_stay_the_classs"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">♣</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">♣</sup>Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3"><text font="italic" xml:id="id3.1">♡</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4">♡</sup>Univ B, City</contact> </creator> <para class="ltx_align_left" xml:id="p1"> <p xml:id="p1.1"><sup xml:id="p1.1.1"><text fontsize="90%" xml:id="p1.1.1.1">♠</text></sup><text fontsize="90%" xml:id="p1.1.2">Univ C, Village</text></p> </para> <para class="ltx_align_left" xml:id="p2"> <p xml:id="p2.1"><sup xml:id="p2.1.1"><text fontsize="90%" xml:id="p2.1.1.1">†</text></sup><text fontsize="90%" xml:id="p2.1.2">Deceased</text></p> </para> <para xml:id="p3"> <p xml:id="p3.1">Text.</p> </para>"#
  );
}

/// "Deceased" naming an institution (`$^\dagger$Department of Deceased Estates, Univ C, Village`) is its carrier's
/// affiliation, not a deceased note: the word is a legend only where the text does not read as an affiliation or opens
/// with it (`legend_note_kind`, `is_legend_text`: `opens_with_deceased`, `reads_as_affiliation`), so the kernel's line
/// route, the kernel's block and REVTeX's binding agree (review rev100a S2, probe e/e1). A legend opening with the word
/// stays its carrier's note though an institution follows (`Deceased 3 March 2021; formerly at Univ C`, probe p/d9).
#[test]
fn deceased_naming_an_institution_is_an_affiliation() {
  assert_eq!(
    converted(
      repro!("run_in_deceased_named_institution_is_an_affiliation"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,†</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id3">†</sup>Department of Deceased Estates, Univ C, Village</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id5">2</sup>Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("run_in_deceased_legend_naming_an_institution_is_a_note"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3"><text font="italic" xml:id="id3.1">2,†</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4">2</sup>Univ B, City</contact> <note class="ltx_note_frontmatter ltx_thanks_deceased" mark="†" role="thanks" xml:id="id5"><sup xml:id="id5.1">†</sup>Deceased 3 March 2021; formerly at Univ C</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}
