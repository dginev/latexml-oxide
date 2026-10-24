//! Red/green guards for batch 79k (complete breakage, continued: user 2026-10-24): pgf's image finds its file, so a
//! TikZ picture its `external` library replaces by the shipped graphic, and a `\pgfimage`, are not missing images.
//! Each guard converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the element
//! whole.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// tikz's `external` library (`mode=graphics if exists`, tikzlibraryexternal_code_tex.rs) includes the shipped
/// `./t-figure0.pdf` for the picture, and `\pgfimage` its `pgfplot.pdf`, through pgfsys-latexml's
/// `\lxSVG@includegraphics`: its candidates are the files the graphics search finds and its box is the file's size
/// (Perl pgfsys-latexml.def.ltxml:597-628). Red (int83a): no candidates, so each was an `ltx_missing_image`. Witnesses
/// 2302.00752 (18 figures), 1905.03839 (12), 1506.05333. Repro package-bindings/tikz_externalized_figure_file.
#[test]
fn an_externalized_tikz_figure_finds_its_file() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/tikz_externalized_figure_file.tex"),
    &[
      (
        "t-figure0.pdf",
        repro!("package-bindings/eps_as_pdf_one.pdf"),
      ),
      ("pgfplot.pdf", repro!("package-bindings/eps_as_pdf_one.pdf")),
    ],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"./t-figure0.pdf\""],
    r#"<graphics candidates="t-figure0.pdf" graphic="./t-figure0.pdf" xml:id="p1.g1"/>"#,
  );
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"pgfplot.pdf\""],
    r#"<graphics candidates="pgfplot.pdf" graphic="pgfplot.pdf" options="width=2cm" xml:id="p2.g1"/>"#,
  );
}

/// comment.sty names its environments `\comment` (`\csarg\def{#1}`, comment.sty:234-248), which a kernel-style
/// `\begin` — arabtex's alatex.sty:113, copied into the repro — looks up: the excluded comment is skipped and an
/// `\includecomment` one kept, as pdflatex prints "A KEPT B". Red (int83a): "Environment comment undefined" and
/// "Environment keep undefined", the comment typeset — the binding (as Perl's) defined only `\begin{comment}`. Witness
/// 2104.08110 (arabtex + comment: a commented-out figure, its 4 images missing). Repro
/// package-bindings/comment_under_kernel_begin.
#[test]
fn a_comment_under_a_kernel_begin_is_skipped() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/comment_under_kernel_begin.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("HIDDEN"), "{xml}");
  assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">A KEPT B</p>"#);
}
