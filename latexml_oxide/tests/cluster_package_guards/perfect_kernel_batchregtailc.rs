//! Red/green guards for batch REGTAILC (row R REGTAIL: papers an old build converted cleanly that later erred or
//! went Fatal): a document only PDF output reads is a pdflatex one whatever its figures say, and a phantom among
//! siblings keeps its height. Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::{assert_element, convert_with};

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

/// math-phantom/phantom_height_among_siblings: a space-like box with height (`\phantom`, `\vphantom`, `\mathstrut`) is
/// a box in its word, as TeX sets latex.ltx's `\finph@nt` box; `\hphantom` has no height (KNOWN_PERL_ERRORS #1327).
/// Each `[wd,ht,dp]` is pdflatex's. Red (INT081G): A, B, D-G and K-M at the siblings' height alone.
#[test]
fn phantom_keeps_its_height_among_siblings() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-phantom/phantom_height_among_siblings.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">A [8.46646pt,6.94444pt,0.0pt].\nB [5.28589pt,6.94444pt,0.0pt].\nC \
     [8.46646pt,4.30554pt,0.0pt].\nD [5.28589pt,7.5pt,2.5pt].\nE [8.05559pt,6.94444pt,0.0pt].\nF \
     [12.50003pt,7.5pt,2.5pt].\nG [13.88893pt,7.5pt,2.5pt].\nH [17.77783pt,4.30554pt,1.94444pt].\nI \
     [13.88892pt,6.94444pt,0.0pt].\nJ [19.57756pt,6.94444pt,0.0pt].\nK [18.46648pt,6.94444pt,0.0pt].\nL \
     [9.60417pt,7.5pt,2.5pt].\nM [8.46646pt,6.94444pt,0.0pt].</p>",
  );
}

/// math-phantom/xy_phantom_object_circle: xy.tex:940 boxes an object as `$\objectstyle{#1}$`, so the circle of
/// `*[o]{\phantom{s}}` is sized by the phantom's height (1210.5701). Red (INT081G): 2 "Dimension too large".
#[test]
fn xy_circled_phantom_object_converts() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-phantom/xy_phantom_object_circle_1210_5701.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<XMHint ").count(), 2, "{xml}");
  assert_eq!(
    xml
      .matches("<XMHint depth=\"0.0pt\" height=\"4.3pt\" name=\"phantom\" width=\"4.7pt\"")
      .count(),
    2,
    "{xml}"
  );
}
