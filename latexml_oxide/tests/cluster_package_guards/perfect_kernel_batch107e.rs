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
//! - authblk prints no note without an `\affil` (authblk.sty:168), so no author requests entry 1 (2510.24320,
//!   2608.24199, 2609.35561; the observer's `Resolver::thenote_finish`).
//!
//! Round 2 (review rev107e):
//! - the observer's writer cuts an entry from inside the font command its mark opens (`emitter::plan::wrapped_entry_mark`),
//!   so a legend line under a wrapped run reaches the legend reader as under a plain run (finding 1, 2605.25195);
//! - REVTeX's head + marked lines are read only where every top-level line of the tail opens with a mark and no box
//!   holds one (`revtex4_support_sty::marks_lead_every_line`), else the 98e fail-safe (finding 2);
//! - a font group whose text goes on after it is no entry's own wrapper: its mark is the entry's text, an isotope's
//!   (`notes::wrapped_mark_at`, finding 3).
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

/// A legend line under a run whose entries each sit in a font command their mark opens (`\textbf{$^1$Univ A}
/// \textbf{$^2$Univ B}\\ $^*$ Equal Contribution, $^\dagger$ Project Leader`): the writer cuts each entry from inside
/// its wrapper, bold kept, so the legend is its carriers' notes, as under a plain run; and the sibling with the legend in
/// a wrapper of its own (`\textit{$^*$Equal Contribution}`), cut with its mark. Distinct from
/// `a_run_is_cut_at_marks_in_groups_or_wrappers` (the kernel's cut): the writer's path. Red (107e r1): Bob and Cy held
/// "Univ B * Equal Contribution, † Project Leader" as affiliation text; (int139a, the sibling) Ann one contact holding it.
/// Witnesses 2605.25195 (its writer declines on its URL line, so it is not fixed by this); the sibling's 2609.38081,
/// 2601.12823 (A/B r2: the legend made its carriers' notes). Repros
/// sectioning-frontmatter/cut_run_legend_line_after_font_wrapped_entries,
/// cut_run_wrapped_legend_line_after_font_wrapped_entries.
#[test]
fn a_legend_after_a_wrapped_run_is_its_carriers_notes() {
  assert_eq!(
    clean_creators(repro!("cut_run_legend_line_after_font_wrapped_entries")),
    r#"<creator role="author"> <personname>Ann Lee</personname> <contact name="Affiliation: " role="affiliation"><text font="bold" xml:id="id1">Univ A</text></contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks" xml:id="id2"><sup xml:id="id2.1">∗</sup> Equal Contribution</note> </creator> <creator before=" " role="author"> <personname>Bob Ray</personname> <contact name="Affiliation: " role="affiliation"><text font="bold" xml:id="id3">Univ B</text></contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks" xml:id="id4"><sup xml:id="id4.1">∗</sup> Equal Contribution</note> </creator> <creator before=" " role="author"> <personname>Cy Do</personname> <contact name="Affiliation: " role="affiliation"><text font="bold" xml:id="id5">Univ B</text></contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id6"><sup xml:id="id6.1">†</sup> Project Leader</note> </creator>"#
  );
}

/// revtex4-2: `\affiliation{Univ A, Nagoya\\ $^{2}$Univ B\\ \parbox{\linewidth}{$^{3}$Inst C\\ $^{4}$Lab D}}` under
/// `Ann$^{1,2}$, Bob$^{2,3}$ and Cy`: the tail after the unmarked head holds boxed marked lines no cut reads, so the whole
/// block is the fail-safe's, once on the block creator (98e), as before 107e. Distinct from
/// `an_unmarked_first_entry_is_its_unprinted_marks` (the read shape, 2109.01501, 1903.07834): its fail-safe edge. Red
/// (107e r1): a nameless creator "Univ A, Nagoya", Ann "Univ B" without entry 1, Bob "Inst C 4 Lab D". Review rev107e
/// finding 2. Repro sectioning-frontmatter/cut_revtex_unmarked_head_unreadable_tail_is_the_blocks.
#[test]
fn an_unmarked_head_over_an_unreadable_tail_is_the_blocks() {
  assert_eq!(
    clean_creators(repro!(
      "cut_revtex_unmarked_head_unreadable_tail_is_the_blocks"
    )),
    r#"<creator role="author"> <personname>Ann Lee<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,2</text></sup></personname> </creator> <creator before=" " role="author"> <personname>Bob Ray<sup xml:id="id2"><text font="italic" xml:id="id2.1">2,3</text></sup></personname> </creator> <creator before=" " role="author"> <personname>Cy Do</personname> </creator> <creator role="author"> <contact name="Affiliation: " role="affiliation">Univ A, Nagoya<break/><sup xml:id="id3"><text font="italic" xml:id="id3.1">2</text></sup>Univ B<break/><inline-block class="ltx_parbox" vattach="middle" width="345.0pt" xml:id="id4"> <p xml:id="id4.1"><sup xml:id="id4.1.1"><text font="italic" xml:id="id4.1.1.1">3</text></sup>Inst C</p> <p xml:id="id4.2"><sup xml:id="id4.2.1"><text font="italic" xml:id="id4.2.1.1">4</text></sup>Lab D</p> </inline-block></contact> </creator>"#
  );
}
