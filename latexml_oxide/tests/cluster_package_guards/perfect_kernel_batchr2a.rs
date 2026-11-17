//! MARKKEY round R2a (brief 2026-11-16_brief_MARKKEY_R2; OD #1061, #1062): a name's request leaves its mark where it
//! printed, keyed by its label, and one settle step after the relocation drops the marks an entry answers. One code
//! path: the kernel's footnote pairing (`relocate_footnote`), which must take no mark the request left for a body
//! `\footnotetext` of its glyph (witnesses 2210.05617, 1304.5040, 1812.11634). The record and the settle step are guarded
//! by the FMRG suite (`perfect_kernel_batchfmmarks`), unchanged.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

use super::perfect_kernel_batchfmmarks::creator_lines;

/// A body `\footnotetext[n]` digested before `\maketitle` stays the body's footnote when a name requests the label n that
/// nothing answers (`$^{1}$`, `$^{2,}$ $^3$`, 2210.05617): the mark the request leaves is no `\footnotemark`, so the
/// kernel's footnote pairing gives it no text, and the settle step, which shows it on the name, takes none away. Red on
/// 3a3b10991a5: the three texts were moved into the marks and lost with them, 0 Warnings.
#[test]
fn a_request_mark_takes_no_body_footnote_text() {
  let (log, xml, ()) = convert_with_setup_then(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/r2a_request_mark_beside_a_body_footnotetext.tex"
    ),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creator_lines(&xml), [
    "Ann Able ; footnotemark[1]=",
    "Bob Baker ; footnotemark[2,3]="
  ]);
  let texts: Vec<(String, String)> = xml
    .match_indices("<note ")
    .filter_map(|(at, _)| {
      let note = &xml[at..at + xml[at..].find("</note>")?];
      let tag = &note[..note.find('>')?];
      if !tag.contains("role=\"footnotetext\"") {
        return None;
      }
      let mark = tag.split(" mark=\"").nth(1)?.split('"').next()?.to_string();
      Some((mark, note[tag.len() + 1..].trim().to_string()))
    })
    .collect();
  assert_eq!(texts, [
    ("1".to_string(), "Univ A, City A.".to_string()),
    ("2".to_string(), "Univ B, City B.".to_string()),
    (
      "3".to_string(),
      "Authors in alphabetical order.".to_string()
    ),
  ]);
}
