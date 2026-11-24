//! Red/green guards for batch REGTAILC (row R REGTAIL: papers an old build converted cleanly that later erred or
//! went Fatal): a document only PDF output reads is a pdflatex one whatever its figures say. Repros under
//! `tools/perfect_kernel/repros/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// An EPS file of a 10 bp square.
const SQUARE_EPS: &str = "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\nnewpath 0 0 moveto 10 0 lineto 10 10 \
                          lineto 0 10 lineto closepath fill\n%%EOF\n";

/// (log, XML) of `tex` converted under ar5iv beside an EPS figure `fig.eps` and its epstopdf conversion.
fn convert_beside_eps(tex: &str) -> (String, String) {
  latexml::util::test::convert_files_with(
    tex,
    &[("fig.eps", SQUARE_EPS), ("fig-eps-converted-to.pdf", "")],
    Some("ar5iv.sty"),
  )
}

/// backend-persona/ifclassinfopdf_unclosed_around_document: `\begin{document}` inside an `\ifCLASSINFOpdf` (or
/// `\ifpdf`) true branch left open is a pdflatex document over its EPS figures' DVI cue, since DVI output would skip
/// the whole text (DG 2026-11-23, OXIDIZED_DESIGN_DIVERGENCES #1154; 2105.10205, 2502.09129). Red (INT081G):
/// `Fatal:Document:Malformed`, no text. The one Warning is TeX's "(\end occurred when \iftrue … incomplete)". The
/// closed template keeps the latex+dvips route (1012.0815's shape).
#[test]
fn unclosed_pdf_switch_around_the_document_is_pdflatex() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/ifclassinfopdf_unclosed_around_document_2105_10205.tex"
  );
  let body = "<p xml:id=\"p1.1\">Body text survives.</p>";
  for tex in [
    tex.to_string(),
    tex
      .replace("\\usepackage{graphicx}", "\\usepackage{graphicx,ifpdf}")
      .replace("\\ifCLASSINFOpdf", "\\ifpdf"),
  ] {
    let (log, xml) = convert_beside_eps(&tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert!(
      log.contains("Attempt to end document with open groups, environments or conditionals"),
      "{log}"
    );
    assert_element(&xml, "p", &[], body);
    assert_element(
      &xml,
      "graphics",
      &[],
      "<graphics candidates=\"fig.eps\" graphic=\"fig.eps\" options=\"width=345.0pt,keepaspectratio=true\" \
       xml:id=\"S0.F1.g1\"/>",
    );
  }
  let closed = tex
    .replace("\\ifCLASSINFOpdf\n", "\\ifCLASSINFOpdf\\else\\fi\n")
    .replace(
      "Body text survives.",
      "Mode: \\ifCLASSINFOpdf PDF\\else DVI\\fi.",
    );
  let (log, xml) = convert_beside_eps(&closed);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "p", &[], "<p xml:id=\"p1.1\">Mode: DVI.</p>");
}
