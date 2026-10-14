//! Red/green guards for batch 66t: a bare `\endX` ends only the frame its own `\X` or `\begin{X}` opened (K9
//! mode-switch study row H, `agent_reports/2026-10-14_k9_mode_switch_study.md`).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// An IEEE Open Journal keywords box (`\vbox\bgroup … \par\endquotation\egroup`) keeps the document body: the bare
/// `\endquotation` has no quotation open and leaves the box's frame alone (one Warning), so the box ends at its
/// `\egroup` and "Body." stays in the document (2301.00752, 2602.19492).
#[test]
fn bare_end_without_its_begin_keeps_the_box() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/bare_end_without_its_begin_keeps_the_box.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("\\endquotation without an environment open"),
    "{log}"
  );
  assert!(xml.contains("Index terms"), "{xml}");
  assert!(xml.contains("Body."), "{xml}");
  assert!(xml.find("Index terms") < xml.find("Body."), "{xml}");
}

/// A bare `\quotation … \endquotation`, and one inside a `\newenvironment` of the document's, still close their
/// quotation: the text after them is outside it.
#[test]
fn bare_quotation_still_ends_its_own() {
  let tex = "\\documentclass{article}\n\\newenvironment{myq}{\\quotation}{\\endquotation}\n\\begin{document}\n\
             \\quotation Quoted.\\endquotation\nAfter one.\n\\begin{myq}Again.\\end{myq}\nAfter two.\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<quote").count(), 2, "{xml}");
  for after in ["After one.", "After two."] {
    let at = xml.find(after).unwrap_or_else(|| panic!("{after}: {xml}"));
    let (opened, closed) = (xml[..at].rfind("<quote"), xml[..at].rfind("</quote>"));
    assert!(closed > opened, "{after} inside the quote: {xml}");
  }
}
