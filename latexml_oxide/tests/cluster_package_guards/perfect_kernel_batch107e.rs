//! Red/green guards for batch 107e: a marked run not cut (census138 row 2, MARKRUN; OXIDIZED_DESIGN_DIVERGENCES #954,
//! amended). A line holding two or more entries, each opened by a mark the names request, is cut at each mark whatever
//! parts them, and a line whose mark prints no label is none:
//! - the shared cutter (`notes::split_marked_pieces`) reads a mark in a group of its own (`{$^2$} B`, 2203.16586) and
//!   one opening its entry's font command (`\textit{$^2$B}`, 2508.07190) as a cut point, drops a separator glyph between
//!   entries (`$|$`), and cuts after a comma with no space (`Lisbon,\IEEEauthorrefmark{2}…`, 2411.03010);
//! - a space before `^` in math is no nucleus (`$ ^1$`, `relocate::spaced_mark_at`: 0902.3613);
//! - a mark opening a line that prints no label (`$^{\;}$`) is no mark: the line continues the entry above
//!   (`affiliations::blank_line_marks_dropped`: 2303.08793);
//! - REVTeX's `\affiliation` with an unmarked first entry over marked lines gives that entry the one requested mark
//!   no line prints and cuts the lines (`revtex4_support_sty::unmarked_head_over_marked_lines`: 2109.01501,
//!   1903.07834);
//! - authblk prints no note without an `\affil` (authblk.sty:167), so no author requests entry 1 (2510.24320,
//!   2608.24199, 2609.35561; the observer's `Resolver::thenote_finish`).
//!
//! Each guard converts its repro and asserts the error and warning counts and every creator whole.
use super::{
  convert_files_with,
  perfect_kernel_batch46::{error_count, warning_count},
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Every creator of `xml`, whitespace squashed.
fn creators(xml: &str) -> String {
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  xml[start..end].to_string()
}

/// Converts `tex`, checks that it errs and warns nothing, and returns its creators.
#[track_caller]
fn clean_creators(tex: &str) -> String {
  let (log, xml) = convert_files_with(tex, &[], Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators(&xml)
}

/// authblk under `\setcounter{Maxaffil}{0}`, the marks written on the `\author` lines and no `\affil`: authblk prints
/// `\AB@author`, no note, so Bob (mark 2) holds Univ B alone. Red (int139a): Bob both entries — the observer read a
/// thenote "1" on every author. Witnesses 2510.24320, 2608.24199, 2609.35561. Repro
/// sectioning-frontmatter/cut_run_authblk_no_affil_marks_its_own_lines.
#[test]
fn authblk_without_affil_prints_no_note() {
  assert_eq!(
    clean_creators(repro!("cut_run_authblk_no_affil_marks_its_own_lines")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator>"#
  );
}

/// One code path, the shared cutter's wrapped marks: a mark in a group of its own (`\small {$^1$} Univ A {$^2$} Univ B
/// {$^3$} Univ C, Paris`, 2203.16586) and one opening each entry's font command, the entries parted by `$|$`
/// (`\textit{$^1$Univ A} $|$ \textit{$^2$Univ B} …`, 2508.07190). Red (int139a): one contact on the first mark's names,
/// the others none, their marks left in the names. Repros sectioning-frontmatter/cut_run_mark_in_a_group_of_its_own,
/// cut_run_font_wrapper_per_entry_bar_separated.
#[test]
fn a_run_is_cut_at_marks_in_groups_or_wrappers() {
  assert_eq!(
    clean_creators(repro!("cut_run_mark_in_a_group_of_its_own")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id1"> Univ A</text></contact> <contact name="Affiliation: " role="affiliation"> Univ B</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id2"> Univ A</text></contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation"> Univ C, Paris</contact> </creator> <creator role="author"> <contact name="URL: " role="url"><ref class="ltx_nolink ltx_url" font="typewriter" fontsize="90%" href="https://example.org/x">https://example.org/x</ref></contact> </creator>"#
  );
  assert_eq!(
    clean_creators(repro!("cut_run_font_wrapper_per_entry_bar_separated")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="id1">Univ A</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="id2">Univ B</text></contact> <contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="id3">Univ C</text></contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="id4">Univ C</text></contact> </creator>"#
  );
}

/// IEEEtran's blockA run `\IEEEauthorrefmark{1}\IEEEauthorrefmark{3}Univ A, Lisbon,\IEEEauthorrefmark{2}Univ B,
/// \IEEEauthorrefmark{1}\IEEEauthorrefmark{2}\IEEEauthorrefmark{3}Inst C` beside a blockA of marked e-mails: the comma
/// before a mark parts the entries. Red (int139a): Ann "Univ A, Lisbon, Univ B" and Inst C, Bob and Cy no affiliation,
/// Cy's e-mail a nameless creator's. Residual pinned: an entry two marks lead is labelled by the first, the second
/// printed as its text. Witness 2411.03010. Repro sectioning-frontmatter/cut_run_ieee_refmarks_after_a_comma.
#[test]
fn a_mark_after_a_comma_parts_the_run() {
  assert_eq!(
    clean_creators(repro!("cut_run_ieee_refmarks_after_a_comma")),
    r#"<creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">‡</sup>Univ A, Lisbon</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id3">†</sup><sup xml:id="id4">‡</sup>Inst C</contact> <contact name="Email: " role="email">ann@a.pt</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id5">†</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id6">†</sup><sup xml:id="id7">‡</sup>Inst C</contact> <contact name="Email: " role="email">bob@b.pt</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole<sup xml:id="id8">‡</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id9">‡</sup>Univ A, Lisbon</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id10">†</sup><sup xml:id="id11">‡</sup>Inst C</contact> <contact name="Email: " role="email">cy@a.pt</contact> </creator>"#
  );
}

/// REVTeX's `\affiliation{$ ^1$Univ A…;  \ $ ^2$Dept B…; $ ^3$Dept C…}` under `$ ^n$` names: a space before `^` in math
/// is skipped, so the call is cut at its marks; and the control, `\affiliation{Dept. A; Univ B, Lisbon}`, one entry
/// (no mark after its `;`). Red (int139a): Ann the whole list, Bob and Cy none. Witness 0902.3613. Repros
/// sectioning-frontmatter/cut_revtex_spaced_math_marks_after_semicolons, cut_revtex_semicolon_one_entry_control.
#[test]
fn a_revtex_call_is_cut_at_its_spaced_marks() {
  assert_eq!(
    clean_creators(repro!("cut_revtex_spaced_math_marks_after_semicolons")),
    r#"<creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Lisbon, Portugal</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">2</sup></personname> <contact name="Affiliation: " role="affiliation">Dept B, Ghent University, Belgium</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole<sup xml:id="id3">3</sup></personname> <contact name="Affiliation: " role="affiliation">Dept C, Univ D, Madrid, Spain</contact> </creator>"#
  );
  assert_eq!(
    clean_creators(repro!("cut_revtex_semicolon_one_entry_control")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Dept. A; Univ B, Lisbon</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Dept. A; Univ B, Lisbon</contact> </creator>"#
  );
}

/// revtex4-2 superscriptaddress: `\affiliation{Univ A, Nagoya, Japan \\ $^2$Univ B, … \\ $^3$Inst C, …}` under
/// `Ann$^{1,2}$, Bob$^{2,3}$ and Cy`: the unmarked first entry is the one requested mark no line prints (1), so Ann's;
/// the marked lines their marks' names. Residual pinned: Cy, on whom revtex prints its own number after the last name,
/// holds nothing, as before. Red (int139a): one nameless creator holding the block. Witnesses 2109.01501, 1903.07834.
/// Repro sectioning-frontmatter/cut_revtex_unmarked_first_entry_over_marked_lines.
#[test]
fn an_unmarked_first_entry_is_its_unprinted_marks() {
  assert_eq!(
    clean_creators(repro!("cut_revtex_unmarked_first_entry_over_marked_lines")),
    r#"<creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,2</text></sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Nagoya, Japan</contact> <contact name="Affiliation: " role="affiliation">Univ B, Tokyo, Japan</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2"><text font="italic" xml:id="id2.1">2,3</text></sup></personname> <contact name="Affiliation: " role="affiliation">Univ B, Tokyo, Japan</contact> <contact name="Affiliation: " role="affiliation">Inst C, Nagakute, Japan</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> </creator>"#
  );
}

/// mnras: `$^{1}$Univ A, 1 Long Street, Waterloo, \\ $^{\;}$Ontario N2L 3G1, Canada \\ $^{2}$Univ B…`: the blank mark
/// labels nothing, so its line continues entry 1. Red (int139a): "Ontario N2L 3G1, Canada" its own contact on the last
/// holder of entry 1. Witness 2303.08793. Repro sectioning-frontmatter/cut_lines_blank_mark_continues_the_entry.
#[test]
fn a_blank_mark_line_continues_the_entry() {
  assert_eq!(
    clean_creators(repro!("cut_lines_blank_mark_continues_the_entry")),
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, 1 Long Street, Waterloo,<break/>Ontario N2L 3G1, Canada</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B, Tokyo, Japan</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation">Univ A, 1 Long Street, Waterloo,<break/>Ontario N2L 3G1, Canada</contact> </creator>"#
  );
}
