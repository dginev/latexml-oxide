//! Guards for batch FB2a (`pk/FB2a`, 2026-11-08): at `LATEXML_FM_EMIT=2` the whole-block fallback is Perl's raw call
//! (`frontmatter/fallback.rs` `whole_block_author`, Base_Utility.pool.ltxml:693), and four of the old reader's rules that
//! its raw-token rewrites carried are ported onto the digested reading (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`,
//! the rows "Strict porting" and "The fallback"; OXIDIZED_DESIGN_DIVERGENCES #52, #980). Each guard is a distinct code
//! path: the queued raw call (`a_raw_queued_call_feeds_no_symbol_mark_to_the_reading`), the rehearsal's binding of a
//! tabular juggle (`a_tabular_juggle_is_a_row_break`), the reading's pre-pass of a trailing alignment environment
//! (`a_trailing_alignment_environment_is_lines_under_the_names`), the rehearsal's text-mode note and the emitter's note
//! call in a formula (`a_note_in_a_formula_keeps_its_words`), and the reading's block-shared URL decided at the block's
//! end (`a_url_line_under_every_name_is_the_blocks`).

use latexml::util::test::{error_count, warning_count};

use super::perfect_kernel_batchz4::{creators_xml, run};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// `tex` converted off and at `=2`, both clean: their creators' XML.
fn both(tex: &str) -> (Vec<String>, Vec<String>) {
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, two) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  (creators_xml(&off), creators_xml(&two))
}

/// The affiliation contact `text`, as both engines write a plain one (its label's space a no-break one).
fn affiliation(text: &str) -> String {
  format!("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{text}</contact>")
}

/// At `=2` the queued call is Perl's raw `\lx@add@author{…}`: the old preparation's `\lx@frontmatter@symbolmark` of
/// `\textsuperscript{*}` no longer feeds the reading, which glued "*Corresponding author" into Bob Baker's affiliation
/// (2608.19750); the note is his, as the PDF prints it under his address. (Off glues it too: the old reader's.)
#[test]
fn a_raw_queued_call_feeds_no_symbol_mark_to_the_reading() {
  let (log, xml) = run(repro!("emit_raw_fallback_symbol_mark_is_no_affiliation"), 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let two = creators_xml(&xml);
  assert_eq!(two.len(), 2, "{two:#?}");
  assert!(
    two[0].contains("<personname>Ann Able</personname>"),
    "{}",
    two[0]
  );
  assert!(two[0].contains(&affiliation("Univ A")), "{}", two[0]);
  let bob = &two[1];
  for part in [
    "<personname>Bob Baker<sup>*</sup></personname>",
    &affiliation("Univ B"),
    "<contact name=\"Email:\u{a0}\" role=\"email\">bob@b.org</contact>",
    "role=\"thanks\"><sup>*</sup>Corresponding author</note>",
  ] {
    assert!(bob.contains(part), "{part} in {bob}");
  }
  assert!(!bob.contains("Univ B *"), "{bob}");
}

/// A parameterless macro closing the title page's tabular and opening another (sig-alternate's
/// `\def\sharedaffiliation{\end{tabular}\begin{tabular}{c}}`) is the row break it prints: at `=2` the rehearsal left the
/// juggle's tabular open and "Dept" in no box, lost with no Warning (0712.1016, 1704.03168, 1301.5612, 1401.6981).
#[test]
fn a_tabular_juggle_is_a_row_break() {
  let (off, two) = both(repro!("author_shared_affiliation_line"));
  assert_eq!(two.len(), 1, "{two:#?}");
  assert!(
    two[0].contains("<personname>A One</personname>"),
    "{}",
    two[0]
  );
  assert!(two[0].contains(&affiliation("Dept")), "{}", two[0]);
  assert_eq!(two, off);
}
