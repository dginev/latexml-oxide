//! MARKKEY round R2a (brief 2026-11-16_brief_MARKKEY_R2; OD #1061, #1062): a name's request leaves its mark where it
//! printed, keyed by its label, and one settle step after the relocation drops the marks an entry answers. One code
//! path: the kernel's footnote pairing (`relocate_footnote`), which must take no mark the request left for a body
//! `\footnotetext` of its glyph (witnesses 2210.05617, 1304.5040, 1812.11634). The record and the settle step are guarded
//! by the FMRG suite (`perfect_kernel_batchfmmarks`), unchanged. Round R2Ab (review revR2A): three paths of the settle step
//! on the footnote-counter decline path (`\footremember`'s `\newcounter`), where it shows every unanswered mark — a
//! picture mark before the label's plain one, a mark read in math, a mark before a `\\` line — one guard each.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

use super::perfect_kernel_batchfmmarks::{creator_lines, page_of};

/// Each creator of `xml` whole, its `xml:id`s dropped and the space between elements cut.
fn creators_whole(xml: &str) -> Vec<String> {
  let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let space = regex::Regex::new(r">\s+<").unwrap();
  xml
    .match_indices("<creator ")
    .map(|(at, _)| {
      let end = at + xml[at..].find("</creator>").expect("closed creator") + "</creator>".len();
      space
        .replace_all(&ids.replace_all(&xml[at..end], ""), "><")
        .into_owned()
    })
    .collect()
}

/// The `before` of a creator after `\and` (article.cls's `\hskip 1em`s) and an author line's `name`, as written.
const AND: &str = "before=\"\u{2003}\u{2003}\"";
const AFFILIATION: &str = "name=\"Affiliation:\u{a0}\"";

/// The log and XML of a sectioning-frontmatter repro, preloading ar5iv.sty.
fn convert(tex: &str) -> (String, String) {
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  (log, xml)
}

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

/// A picture mark (`$^{\circled{b}}$`: its label known, its glyph not) is dropped from the name without taking its label,
/// so the plain mark of that label after it shows: in its own superscript (`$^{\circled{b}}$$^{b}$`), inside a glued run
/// (`$^{1,\circled{b}}$$^{b}$`) and after a plain mark (`$^{2}$$^{\circled{c}}$$^{c}$`). Red on 1012ac10283: the picture
/// took the label and the plain b and c were dropped as "again", 0 Warnings (review revR2A B1).
#[test]
fn a_picture_mark_takes_no_label_from_the_plain_mark_after_it() {
  let thanks = r#"<note class="ltx_note_frontmatter ltx_thanks_note" mark="*" role="thanks">Zebra University.</note>"#;
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/r2a_picture_mark_before_the_plain_mark_of_its_label.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(creators_whole(&xml), [
    format!(
      r#"<creator role="author"><personname>Ann Able</personname><note mark="b" role="footnotemark"/>{thanks}</creator>"#
    ),
    format!(
      r#"<creator {AND} role="author"><personname>Bob Baker</personname><note mark="c" role="footnotemark"/></creator>"#
    ),
  ]);
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/r2a_picture_mark_in_a_glued_run_before_its_plain_mark.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(creators_whole(&xml), [
    format!(
      r#"<creator role="author"><personname>Ann Able</personname><note mark="b" role="footnotemark"/>{thanks}<contact class="ltx_author_line" {AFFILIATION} role="affiliation">Dept. of B</contact></creator>"#
    ),
    format!(
      r#"<creator {AND} role="author"><personname>Bob Baker<break/>Univ. of C</personname><note mark="2,c" role="footnotemark"/></creator>"#
    ),
  ]);
}

/// A name's request read in math leaves its mark after the math (`\aftergroup`) and reverts to nothing: `$\star^{1}$`'s
/// `tex` is `\star` (no internal macro in it nor in the page's `alttext`), `$^{2}_{x}$`'s script keeps the base's text
/// `<sub>x</sub>`, and a mark after the math before a `\\` line shows after the name. Red on 1012ac10283: `tex="\star
/// \lx@frontmatter@namemark{affiliation:1}{1}"`, Bob's `x` an empty-base `POSTSUBSCRIPT` (review revR2A B2).
#[test]
fn a_request_mark_read_in_math_waits_out_the_math() {
  let star = r#"<Math mode="inline" tex="\star" text="star"><XMath><XMTok name="star" role="MULOP">⋆</XMTok></XMath></Math>"#;
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/r2a_request_mark_in_math_prints_nothing_there.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  assert_eq!(creators_whole(&xml), [
    format!(
      r#"<creator role="author"><personname>Ann Able{star}</personname><note mark="1" role="footnotemark"/><note class="ltx_note_frontmatter ltx_thanks_note" mark="*" role="thanks">Zebra University.</note></creator>"#
    ),
    format!(
      r#"<creator {AND} role="author"><personname>Bob Baker<sub><text font="italic">x</text></sub></personname><note mark="2" role="footnotemark"/></creator>"#
    ),
    format!(
      r#"<creator {AND} role="author"><personname>Cat Cole</personname><note mark="3" role="footnotemark"/></creator>"#
    ),
  ]);
  let page = page_of(&xml);
  assert!(!page.contains("lx@") && !xml.contains("lx@"), "{page}");
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/r2a_request_mark_in_math_before_a_name_line.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(
    creators_whole(&xml)[1],
    format!(
      r#"<creator {AND} role="author"><personname>Bob Baker{star}<break/>Univ. of C</personname><note mark="2" role="footnotemark"/></creator>"#
    )
  );
  assert!(!xml.contains("lx@"), "{xml}");
}

/// A name's unanswered marks printed before more of the name (`Bob Baker$^{2}$$^{3}$\footnotemark[1] \\ Univ.\ of C`)
/// show after the name, joined as the binding's are, so the pure-name cut sets them after it: the name stays pure. Red
/// on 1012ac10283: `<sup>2</sup><sup>3</sup>` inside the name before its `<break/>` (review revR2A S1).
#[test]
fn a_mark_before_a_name_line_shows_after_the_name() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/r2a_name_marks_before_a_name_line_show_after_it.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(creators_whole(&xml), [
    format!(
      r#"<creator role="author"><personname>Ann Able</personname><note mark="∗" role="footnotemark"/><note class="ltx_note_frontmatter ltx_thanks_note" mark="*" role="thanks">Zebra University.</note><contact class="ltx_author_line" {AFFILIATION} role="affiliation">Dept. of B</contact></creator>"#
    ),
    format!(
      r#"<creator {AND} role="author"><personname>Bob Baker<break/>Univ. of C</personname><note mark="2,3" role="footnotemark"/><note class="ltx_note_frontmatter ltx_thanks_note" idref="id1" mark="*" role="thanks">Zebra University.</note></creator>"#
    ),
  ]);
}
