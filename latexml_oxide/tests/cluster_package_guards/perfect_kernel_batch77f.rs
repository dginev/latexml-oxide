//! Red/green guard for batch 77f: a paragraph break inside a frontmatter item whose element holds only inline content
//! is a line break of the item (user ruling 2026-10-18, OXIDIZED_DESIGN_DIVERGENCES #766; base_utilities.rs
//! `frontmatter_paragraph_break`, decided at construction by the schema, and `paragraph_breaks_between_content`). The
//! guards assert the three measures: no Error, the authored text present once, each paragraph its own line.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// The acknowledgements' paragraphs (`Inline.model`) are lines, in a group of the item too; breaks at its ends and a
/// run of them add nothing; the abstract (`Block.model`) keeps its paragraphs.
#[test]
fn inline_item_paragraphs_are_line_breaks() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/inline_item_paragraphs_are_line_breaks.tex"
    ),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<acknowledgements inlist="toc" name="Acknowledgements" xml:id="acknowledgements1"> First part.<break/>Second part.<break/><text font="bold">Bold part.<break/>Bold again.</text> </acknowledgements>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p>First paragraph.</p> <p>Second paragraph.</p> </abstract>"#),
    "{xml}"
  );
  for text in [
    "First part.",
    "Second part.",
    "Bold part.",
    "Bold again.",
    "First paragraph.",
    "Second paragraph.",
  ] {
    assert_eq!(xml.matches(text).count(), 1, "{text}: {xml}");
  }
}

/// apa.cls's two-paragraph `\acknowledgements` is two lines of its "Author Note" acknowledgements, in every mode.
#[test]
fn apa_acknowledgements_paragraphs_are_lines() {
  if !latexml::util::test::kpse_has("apa.cls") {
    eprintln!("skipped: apa.cls is not installed");
    return;
  }
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/apa_acknowledgements_paragraphs_are_lines.tex"
  );
  assert!(
    tex.contains("[doc]{apa}"),
    "the mode replacements read the repro's class line"
  );
  for mode in ["doc", "jou", "man"] {
    let (log, xml) = convert_with(
      &tex.replace("[doc]{apa}", &format!("[{mode}]{{apa}}")),
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 1),
      "{mode}: {log}"
    );
    let xml = squash(&xml);
    assert!(
      xml.contains(r#"<acknowledgements inlist="toc" name="Author Note" xml:id="acknowledgements1">We thank the funding agency.<break/>Correspondence to A. Author.</acknowledgements>"#),
      "{mode}: {xml}"
    );
    assert_eq!(
      xml.matches("Correspondence to A. Author.").count(),
      1,
      "{mode}: {xml}"
    );
  }
}
