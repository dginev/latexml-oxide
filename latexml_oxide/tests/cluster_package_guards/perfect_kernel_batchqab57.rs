//! Guards for batch QAB57 (three Bruce Q&A residuals): under OmniBus the document's definitions win over its guesses
//! (QA58).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// The body of `<document …>`, from its first child on, with the stylesheet resources dropped.
fn body(xml: &str) -> String {
  let start = xml.find("<document").unwrap_or(0);
  let rest = &xml[start..];
  let rest = &rest[rest.find('>').map_or(0, |i| i + 1)..];
  rest
    .lines()
    .filter(|l| !l.trim_start().starts_with("<resource "))
    .collect::<Vec<_>>()
    .join("\n")
}

/// Under OmniBus (a class with no binding) the document's own definitions of its locked frontmatter guesses win, every
/// definer of the `\newcommand` family and `\newenvironment` (BM Q3.12, DG 2026-11-14); a package's begin-document hook
/// is not the document, and apacite's `\providecommand{\doi}` keeps OmniBus's link (user ruling 2026-10-01).
#[test]
fn omnibus_guesses_yield_to_the_document() {
  let rows: [(&str, &str); 2] = [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/singletons/omnibus_document_redefines_frontmatter_guesses.tex"
      ),
      "\n  <para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">X E:a@b K:k A:ad O:0 D:10 F:u.\nK:k.</p>\n  </para>\n</document>",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/singletons/omnibus_package_hook_keeps_guessed_doi.tex"
      ),
      "\n  <para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">X <ref href=\"https://doi.org/10.1000/xyz\">10.1000/xyz</ref>.</p>\n  </para>\n</document>",
    ),
  ];
  for (tex, expected) in rows {
    let (log, xml) = convert_counted(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert_eq!(body(&xml), expected, "{xml}");
  }
}
