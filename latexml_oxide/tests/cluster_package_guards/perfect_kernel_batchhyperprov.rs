//! Red/green guards for HYPERPROV: the ar5iv preload's arXiv-hyperref emulation (OXIDIZED_DESIGN #588) provides
//! `\texorpdfstring` and atbegshi's shipout hooks at `\begin{document}` to a paper that never loaded hyperref or
//! atbegshi, only the names still undefined (DG ruling 2026-11-19).
use super::perfect_kernel_batch46::{error_count, warning_count};

const TEXORPDFSTRING: &str = include_str!(
  "../../../tools/perfect_kernel/repros/package-bindings/texorpdfstring_without_hyperref.tex"
);
const ATBEGSHI: &str = include_str!(
  "../../../tools/perfect_kernel/repros/package-bindings/atbegshi_hooks_without_package.tex"
);

fn title(xml: &str) -> &str {
  let start = xml.find("<title>").expect("a section title");
  &xml[start..start + xml[start..].find("</title>").unwrap() + "</title>".len()]
}

/// hyperref's `\texorpdfstring` keeps its TeX string in a section title and drops the PDF string, which was printed
/// after an `undefined` Error (hep-th0702068).
#[test]
fn texorpdfstring_without_hyperref() {
  let (log, xml) = latexml::util::test::convert_with(TEXORPDFSTRING, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let title = title(&xml);
  assert!(
    title.starts_with(r#"<title><tag close=" ">1</tag>Energy <Math mode="inline" tex="E=mc^{2}""#)
      && title.ends_with("</Math> law</title>")
      && title.matches("<Math ").count() == 1,
    "{title}"
  );
  assert!(!xml.contains("mc2") && !xml.contains("ERROR"), "{xml}");
}

/// Provide-only: a paper's own `\texorpdfstring` (here keeping the PDF string) stays its own.
#[test]
fn own_texorpdfstring_kept() {
  let tex = "\\documentclass{article}\n\\newcommand{\\texorpdfstring}[2]{#2}\n\\begin{document}\n\
             \\section{A \\texorpdfstring{$x$}{ex} b}\n\\end{document}\n";
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    title(&xml),
    r#"<title><tag close=" ">1</tag>A ex b</title>"#
  );
}

/// atbegshi's hooks take their material and print none of it; `\AtBeginShipoutBox` is a box register
/// (2309.06611, 2403.15239, 1607.05658, 2507.04353).
#[test]
fn atbegshi_hooks_without_package() {
  let (log, xml) = latexml::util::test::convert_with(ATBEGSHI, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">Body text.</p></para>"#,
  );
  assert_eq!(xml.matches("<para").count(), 1, "{xml}");
}

/// Without the ar5iv preload nothing is provided: both documents keep TeX's `undefined` Errors.
#[test]
fn no_provide_without_the_preload() {
  let (log, _xml) = latexml::util::test::convert_with(TEXORPDFSTRING, None);
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("Error:undefined:\\texorpdfstring"), "{log}");
  let (log, _xml) = latexml::util::test::convert_with(ATBEGSHI, None);
  assert_eq!((error_count(&log), warning_count(&log)), (9, 2), "{log}");
  assert!(log.contains("Error:undefined:\\AtBeginShipoutBox"), "{log}");
}
