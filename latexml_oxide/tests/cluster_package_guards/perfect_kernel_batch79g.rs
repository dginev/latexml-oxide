//! Red/green guards for batch 79g (complete breakage, user 2026-10-24: a missing image, a broken `\ref` or `\cite` is a
//! readability failure held to 100 %; the fixes are coverage gaps). Each guard converts its repro under
//! `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the element whole.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// A graphic named without its directory is found under an `\input@path` entry, as pdflatex finds it (graphics.sty's
/// `\Gin@getbase` tests each name with `\IfFileExists`, which looks under every `\input@path` entry, latex.ltx:9698
/// `\@iffileonpath`). Red (int79a): "No graphic source found", an `ltx_missing_image` in the HTML — the image search
/// took `\graphicspath`, the search paths and the source directory only. Witness 1112.0840 (8 figures in `graphics/`
/// under its class's `\csdef{input@path}{{style/}{graphics/}{}}`); the box is sized from the file found. Repro
/// package-bindings/includegraphics_on_input_path.
#[test]
fn includegraphics_looks_under_the_input_path() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/includegraphics_on_input_path.tex"),
    &[(
      "figs/ip_plot.pdf",
      repro!("package-bindings/eps_as_pdf_one.pdf"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"ip_plot\""],
    r#"<graphics candidates="figs/ip_plot.pdf" cssstyle="width:20.075em; height:10.037em" graphic="ip_plot" xml:id="p1.g1"/>"#,
  );
}
