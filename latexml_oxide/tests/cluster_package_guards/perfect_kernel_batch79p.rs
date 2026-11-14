//! Red/green guards for batch 79p (complete breakage, continued: user 2026-10-24): array.sty's preamble builder
//! (`\@mkpream` and its column classes) for raw code that builds its own alignment preamble with it. Each guard
//! converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// An lhs2TeX code block on polytable.sty in a figure: polytable builds each column's one-column preamble with
/// array.sty's `\@mkpream` (array_sty.rs) and splits it at `\@sharp` into the material around a cell, so the three code
/// lines print, each its own formula on its own line, and the figure's label resolves. Red (int87b): "undefined
/// `\@classiv`"/"`\@classz`" from the kernel's `\@mkpream`, polytable's "Illegal preamble (no columns)", a Fatal at the
/// end of the file, the figure lost. Witnesses 2307.11194, 1705.09704. Repro package-bindings/polytable_code_block.
#[test]
fn polytable_code_block_keeps_its_lines() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/polytable_code_block.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let figure = xml
    .split("<figure")
    .nth(1)
    .and_then(|f| f.split("</figure>").next())
    .unwrap_or_default();
  assert_eq!(figure.matches("<Math ").count(), 3, "{xml}");
  assert_eq!(figure.matches("<break").count(), 2, "{xml}");
  for keyword in [">data</XMTok>", ">instance</XMTok>", ">where</XMTok>"] {
    assert_eq!(figure.matches(keyword).count(), 1, "{keyword}: {xml}");
  }
  assert!(
    xml.contains(r#"<ref labelref="LABEL:fig:greet"/>"#),
    "{xml}"
  );
}
