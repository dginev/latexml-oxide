//! Red/green guards for batch 66r: two quick wins from the K9 mode-switch study
//! (`agent_reports/2026-10-14_k9_mode_switch_study.md`, rows P and T).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// emlines' `\emline` is two `\put`s and a `\special`, balanced: the picture closes where the document closes it and
/// the text after it stays outside (0911.1421, cond-mat0106468).
#[test]
fn emline_in_picture_is_balanced() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/emline_in_picture_is_balanced.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let after_picture = xml
    .split("</picture>")
    .nth(1)
    .unwrap_or_else(|| panic!("no picture: {xml}"));
  assert!(after_picture.contains("After."), "{xml}");
}

/// A `\twocolumn[…]` header spanning the document's own conditional keeps both halves of it: the header is typeset
/// once and the body follows (1810.09262, 1912.04806).
#[test]
fn twocolumn_header_spans_a_conditional() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/twocolumn_header_spans_a_conditional.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("Title").count(), 1, "{xml}");
  assert_eq!(xml.matches("Body.").count(), 1, "{xml}");
  assert!(xml.find("Title") < xml.find("Body."), "{xml}");
}

/// The empty and absent headers stay a bare paragraph break.
#[test]
fn twocolumn_without_header() {
  let tex = "\\documentclass{article}\n\\begin{document}\nA.\n\\twocolumn\nB.\n\\twocolumn[]\nC.\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<p ").count(), 3, "{xml}");
}
