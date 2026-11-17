//! SHCOPY (DG 2026-11-17, "Copy in HTML only"; OD #1052): a shared author note stays one note in the XML, its copies
//! `idref` to the first (OD #1059), and in the HTML every other mark carries the note's pop-up, hidden
//! (`span.ltx_note_outer.ltx_note_copy`, `aria-hidden`, no id, links out of the tab order), so on a narrow screen each
//! mark pops the note up, not the first alone. Witnesses 2407.07614, 2606.00446, 1706.03762, 2506.06941. The page layer
//! of every FMRG row checks each linked mark's copy (`perfect_kernel_batchfmmarks::page_lines`); the XSLT's branches
//! are `latexml_post`'s `integration::a_shared_notes_other_marks_carry_its_hidden_copy`.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

use super::perfect_kernel_batchfmmarks::{page_lines, page_of};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The XML's note start tags, and the page's hidden copies (each `span.ltx_note_copy` whole).
fn notes_and_copies(tex: &str) -> (Vec<String>, Vec<String>) {
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let notes = xml
    .match_indices("<note ")
    .map(|(at, _)| xml[at..at + xml[at..].find('>').unwrap() + 1].to_string())
    .collect();
  let html = page_of(&xml);
  let (_, problems) = page_lines(&html);
  assert_eq!(problems, Vec::<String>::new(), "{html}");
  let copies = html
    .match_indices(r#"<span class="ltx_note_outer ltx_note_copy""#)
    .map(|(at, _)| {
      let end = at + html[at..].find("</span></span>").unwrap() + "</span></span>".len();
      html[at..end].to_string()
    })
    .collect();
  (notes, copies)
}

/// One note per shared note in the XML, its copies naming it by `idref`; on the page each copy's mark carries it hidden.
/// Shapes: 2407.07614's legend `∗` (two carriers), 1706.03762's `\footnotemark` leftovers (four carriers, two with a
/// note of their own), a shared unmarked note under the page's `[a]` (OD #1059).
/// A copy of the note `text` under `mark` (`sup` the mark's class), as the page writes it.
fn copy(sup: &str, mark: &str, text: &str) -> String {
  format!(
    r#"<span class="ltx_note_outer ltx_note_copy" aria-hidden="true"><span class="ltx_note_content"><sup class="{sup}">{mark}</sup><span class="ltx_note_type">thanks: </span>{text}</span></span>"#
  )
}

/// One note per shared note in the XML, its copies naming it by `idref`; on the page each copy's mark carries it hidden.
/// Shapes: 2407.07614's legend `∗` (two carriers), 1706.03762's `\footnotemark` leftovers (four carriers, two with a
/// note of their own), a shared unmarked note under the page's `[a]` (OD #1059).
#[test]
fn a_shared_note_is_one_note_in_the_xml_and_each_mark_pops_it_up() {
  const THANKS: &str = r#"<note class="ltx_note_frontmatter ltx_thanks_contribution""#;
  // Before (MTREDUCEd): the same XML; on the page each copy its linked mark alone.
  assert_eq!(
    notes_and_copies(repro!(
      "author_block_symbol_legend_note_mark_not_in_its_text"
    )),
    (
      vec![
        format!(r#"{THANKS} mark="∗" role="thanks" xml:id="id1">"#),
        format!(r#"{THANKS} idref="id1" mark="∗" role="thanks" xml:id="id2">"#),
      ],
      vec![copy("ltx_note_mark", "∗", "Equal contribution")]
    )
  );
  let long = "Equal contribution. Listing order is random. Author A designed part 1 of the model and ran the \
              experiments of section 1. Author B designed part 2 of the model and ran the experiments of section 2. \
              Author C designed part 3 of the model and ran the experiments of section 3. Author D designed part 4 of \
              the model and ran the experiments of section 4. Author E designed part 5 of the model and ran the \
              experiments of section 5. Author F designed part 6 of the model and ran the experiments of section 6. \
              Author G designed part 7 of the model and ran the experiments of section 7. Author H designed part 8 of \
              the model and ran the experiments of section 8.";
  assert_eq!(
    notes_and_copies(repro!("fmmarks_long_shared_note_with_second_marks")),
    (
      vec![
        format!(r#"{THANKS} mark="*" role="thanks" xml:id="id1">"#),
        format!(r#"{THANKS} idref="id1" mark="*" role="thanks" xml:id="footnotex1">"#),
        format!(r#"{THANKS} idref="id1" mark="*" role="thanks" xml:id="footnotex2">"#),
        r#"<note class="ltx_note_frontmatter ltx_thanks_note" mark="†" role="thanks" xml:id="id4">"#.to_string(),
        format!(r#"{THANKS} idref="id1" mark="*" role="thanks" xml:id="footnotex3">"#),
        r#"<note class="ltx_note_frontmatter ltx_thanks_note" mark="‡" role="thanks" xml:id="id6">"#.to_string(),
      ],
      vec![copy("ltx_note_mark", "*", long); 3]
    )
  );
  assert_eq!(
    notes_and_copies(repro!(
      "markkey_footnotemark_shares_a_thanks_by_number_unbound_class"
    )),
    (
      vec![
        format!(r#"{THANKS} role="thanks" xml:id="id1">"#),
        format!(r#"{THANKS} idref="id1" role="thanks" xml:id="footnotex1">"#),
      ],
      vec![copy(
        "ltx_note_mark ltx_mark_generated",
        "[a]",
        "Equal contribution."
      )]
    )
  );
}
