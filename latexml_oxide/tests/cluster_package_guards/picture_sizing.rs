//! An `ltx:picture` that is auto-opened (by a drawing object outside any
//! explicit picture) takes its size from the box that opened it, as Perl's tag
//! `afterClose` does (latex_constructs.pool.ltxml:4943-4950): width, and height
//! plus depth, in px, unless the picture already has them. An unsized
//! picture is drawn by a browser as a 300×150 box, flipped. Batch 56iy; repro
//! `tools/perfect_kernel/repros/graphics-tikz/pspicture_inner_box_picture_size.tex`
//! (pst-flags-doc: 517 unsized pictures; egameps: 162).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// Every `<picture>` carries a width and a height; the nested one is 0×0 like
/// the `\psframe` that opened it, as Perl writes it (`width="0" height="0"`),
/// and a `\put(0,0){Hello}` in a paragraph is sized as Perl sizes it
/// (31.13×9.61 px).
#[test]
fn auto_opened_picture_is_sized_from_its_box() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pspicture_inner_box_picture_size.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let pictures: Vec<&str> = xml
    .match_indices("<picture")
    .map(|(i, _)| &xml[i..])
    .collect();
  assert_eq!(pictures.len(), 2, "{xml}");
  for picture in pictures {
    let open = &picture[..picture.find('>').unwrap_or(picture.len())];
    assert!(
      open.contains(" width=\"") && open.contains(" height=\""),
      "unsized: {open}\n{xml}"
    );
  }
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p1.pic1.pic1""#],
    r#"<picture height="0" width="0" xml:id="p1.pic1.pic1"><rect fill="none" height="39.37" stroke="black" stroke-width="0.8" width="39.37" x="0" y="0"/></picture>"#,
  );
  let (stderr, xml) = convert(
    "\\documentclass{article}\n\\begin{document}\n\\put(0,0){Hello}\n\\end{document}\n",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let open = xml
    .find("<picture")
    .map(|i| &xml[i..i + xml[i..].find('>').unwrap_or(0)]);
  assert_eq!(
    open,
    Some(r#"<picture height="9.61" width="31.13" xml:id="p1.pic1""#),
    "{xml}"
  );
}

/// A minipage in SVG keeps its TeX width — 200pt, 100pt, tcolorbox's 168.7pt `\linewidth` (pdflatex) — as ems of its
/// foreignObject's `font-size` anchor, whatever font its body ends in or its node is set in: italic, typewriter and
/// sans last boxes, a `\scriptsize` node, a `[font=\scriptsize]` picture, a `\ttfamily` tcolorbox, `fontupper=\small`.
/// Repro `graphics-tikz/svg_block_width_font_size_ems.tex` (2605.01325, 2605.07905).
#[test]
fn svg_block_width_is_in_font_size_ems() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/svg_block_width_font_size_ems.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p1.pic1""#],
    r##"<picture height="9.61" width="276.74" xml:id="p1.pic1"><svg:svg height="9.61" overflow="visible" version="1.1" viewBox="0 0 276.74 9.61" width="276.74"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,9.61) matrix(1 0 0 -1 0 0) translate(138.37,0) translate(0,4.8) matrix(1.0 0.0 0.0 1.0 -138.37 -4.8)"><svg:foreignObject height="9.61" overflow="visible" style="--ltx-fo-width:20em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="276.74"><inline-block class="ltx_minipage" vattach="bottom" width="20em"><p><text font="italic">What makes</text></p></inline-block></svg:foreignObject></svg:g></svg:svg></picture>"##,
  );
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p1.pic2""#],
    r##"<picture height="8.46" width="276.74" xml:id="p1.pic2"><svg:svg height="8.46" overflow="visible" version="1.1" viewBox="0 0 276.74 8.46" width="276.74"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,8.46) matrix(1 0 0 -1 0 0) translate(138.37,0) translate(0,4.23) matrix(1.0 0.0 0.0 1.0 -138.37 -4.23)"><svg:foreignObject height="8.46" overflow="visible" style="--ltx-fo-width:20em;--ltx-fo-height:0.61em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 8.46)" width="276.74"><inline-block class="ltx_minipage" vattach="bottom" width="20em"><p><text font="typewriter">What makes</text></p></inline-block></svg:foreignObject></svg:g></svg:svg></picture>"##,
  );
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p1.pic3""#],
    r##"<picture height="9.61" width="276.74" xml:id="p1.pic3"><svg:svg height="9.61" overflow="visible" version="1.1" viewBox="0 0 276.74 9.61" width="276.74"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,9.61) matrix(1 0 0 -1 0 0) translate(138.37,0) translate(0,4.8) matrix(1.0 0.0 0.0 1.0 -138.37 -4.8)"><svg:foreignObject height="9.61" overflow="visible" style="--ltx-fo-width:20em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="276.74"><inline-block class="ltx_minipage" vattach="bottom" width="20em"><p><text font="sansserif">What makes</text></p></inline-block></svg:foreignObject></svg:g></svg:svg></picture>"##,
  );
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p2.pic1""#],
    r##"<picture height="6.73" width="138.37" xml:id="p2.pic1"><svg:svg height="6.73" overflow="visible" version="1.1" viewBox="0 0 138.37 6.73" width="138.37"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,6.73) matrix(1 0 0 -1 0 0) translate(69.19,0) translate(0,3.36) matrix(1.0 0.0 0.0 1.0 -69.19 -3.36)"><svg:foreignObject height="6.73" overflow="visible" style="--ltx-fo-width:14.29em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:7pt;" transform="matrix(1 0 0 -1 0 6.73)" width="138.37"><inline-block class="ltx_minipage" vattach="top" width="14.29em"><p/><p class="ltx_align_left"><text fontsize="70%">What makes</text></p></inline-block></svg:foreignObject></svg:g></svg:svg></picture>"##,
  );
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p3.pic1""#],
    r##"<picture height="6.73" width="138.37" xml:id="p3.pic1"><svg:svg height="6.73" overflow="visible" version="1.1" viewBox="0 0 138.37 6.73" width="138.37"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,6.73) matrix(1 0 0 -1 0 0) translate(69.19,0) translate(0,3.36) matrix(1.0 0.0 0.0 1.0 -69.19 -3.36)"><svg:foreignObject height="6.73" overflow="visible" style="--ltx-fo-width:10em;--ltx-fo-height:0.49em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 6.73)" width="138.37"><inline-block class="ltx_minipage" vattach="top" width="10em"><p/><p class="ltx_align_left"><text fontsize="70%">What makes</text></p></inline-block></svg:foreignObject></svg:g></svg:svg></picture>"##,
  );
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p4.pic1""#],
    r##"<picture height="39.09" width="276.74" xml:id="p4.pic1"><svg:svg height="39.09" overflow="visible" version="1.1" viewBox="0 0 276.74 39.09" width="276.74"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,39.09) matrix(1 0 0 -1 0 0)"><svg:g fill="#404040" fill-opacity="1.0"><svg:path d="M 0 5.91 L 0 33.18 C 0 36.45 2.64 39.09 5.91 39.09 L 270.83 39.09 C 274.1 39.09 276.74 36.45 276.74 33.18 L 276.74 5.91 C 276.74 2.64 274.1 0 270.83 0 L 5.91 0 C 2.64 0 0 2.64 0 5.91 Z" style="stroke:none"/></svg:g><svg:g fill="#F2F2F2" fill-opacity="1.0"><svg:path d="M 1.97 5.91 L 1.97 33.18 C 1.97 35.36 3.73 37.12 5.91 37.12 L 270.83 37.12 C 273.01 37.12 274.77 35.36 274.77 33.18 L 274.77 5.91 C 274.77 3.73 273.01 1.97 270.83 1.97 L 5.91 1.97 C 3.73 1.97 1.97 3.73 1.97 5.91 Z" style="stroke:none"/></svg:g><svg:g fill-opacity="1.0" transform="matrix(1.0 0.0 0.0 1.0 21.65 16.85)"><svg:foreignObject height="11.53" overflow="visible" style="--ltx-fo-width:16.07em;--ltx-fo-height:0.58em;--ltx-fo-depth:0.21em;font-size:10.5pt;" transform="matrix(1 0 0 -1 0 8.46)" width="233.43"><inline-block class="ltx_minipage" vattach="bottom" width="16.07em"><p><text color="#000000" font="typewriter">Typewriter box.</text></p></inline-block></svg:foreignObject></svg:g></svg:g></svg:svg></picture>"##,
  );
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p5.pic1""#],
    r##"<picture height="38.63" width="276.74" xml:id="p5.pic1"><svg:svg height="38.63" overflow="visible" version="1.1" viewBox="0 0 276.74 38.63" width="276.74"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,38.63) matrix(1 0 0 -1 0 0)"><svg:g fill="#404040" fill-opacity="1.0"><svg:path d="M 0 5.91 L 0 32.72 C 0 35.98 2.64 38.63 5.91 38.63 L 270.83 38.63 C 274.1 38.63 276.74 35.98 276.74 32.72 L 276.74 5.91 C 276.74 2.64 274.1 0 270.83 0 L 5.91 0 C 2.64 0 0 2.64 0 5.91 Z" style="stroke:none"/></svg:g><svg:g fill="#F2F2F2" fill-opacity="1.0"><svg:path d="M 1.97 5.91 L 1.97 32.72 C 1.97 34.9 3.73 36.66 5.91 36.66 L 270.83 36.66 C 273.01 36.66 274.77 34.9 274.77 32.72 L 274.77 5.91 C 274.77 3.73 273.01 1.97 270.83 1.97 L 5.91 1.97 C 3.73 1.97 1.97 3.73 1.97 5.91 Z" style="stroke:none"/></svg:g><svg:g fill-opacity="1.0" transform="matrix(1.0 0.0 0.0 1.0 21.65 16.2)"><svg:foreignObject height="11.07" overflow="visible" style="--ltx-fo-width:16.87em;--ltx-fo-height:0.63em;--ltx-fo-depth:0.18em;font-size:10pt;" transform="matrix(1 0 0 -1 0 8.65)" width="233.43"><inline-block class="ltx_minipage" vattach="bottom" width="16.87em"><p><text color="#000000" fontsize="90%">Small body.</text></p></inline-block></svg:foreignObject></svg:g></svg:g></svg:svg></picture>"##,
  );
}

/// pgf evaluates `em`, `ex` and `width("…")` in the document's font, as `\pgfmath@selectfont` selects it, not the
/// picture's `\nullfont`: `8em` at `\footnotesize` is pdflatex's 68.00098pt, a `text width=8em` node's frame 74.06714pt
/// (pdflatex 74.06711pt), `4ex` at `\Large\itshape` 24.79999pt, `width("xxxx")` 17.9447 and a `minimum
/// width={width(…)}` node 36.2894pt. Repro `graphics-tikz/pgfmath_em_selectfont.tex` (row 2, a raw `8em` under
/// `\nullfont`, RED).
#[test]
fn pgfmath_em_is_the_document_fonts() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_em_selectfont.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><picture height="1" width="1" xml:id="p1.pic1"><svg:svg height="1" overflow="visible" version="1.1" viewBox="0 0 1 1" width="1"><svg:g transform="translate(0,1) matrix(1 0 0 -1 0 0)"/></svg:svg></picture><p><text fontsize="80%">[1:68.00098pt]</text></p></para>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r##"<para xml:id="p3"><picture height="13.31" width="102.49" xml:id="p3.pic1"><svg:svg height="13.31" overflow="visible" version="1.1" viewBox="0 0 102.49 13.31" width="102.49"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,13.31) matrix(1 0 0 -1 0 0) translate(51.24,0) translate(0,6.66)"><svg:path d="M -50.97 -6.38 h 101.93 v 12.76 h -101.93 Z" style="fill:none"/><svg:g fill="#000000" stroke="#000000" transform="matrix(1.0 0.0 0.0 1.0 -47.05 -2.46)"><svg:foreignObject height="4.92" overflow="visible" style="--ltx-fo-width:8.5em;--ltx-fo-height:0.44em;--ltx-fo-depth:0em;font-size:8pt;" transform="matrix(1 0 0 -1 0 4.92)" width="94.09"><inline-block class="ltx_minipage" vattach="top" width="8.5em"><p/><p class="ltx_align_left"><text font="sansserif" fontsize="80%">x</text></p></inline-block></svg:foreignObject></svg:g></svg:g></svg:svg></picture><p><text font="sansserif" fontsize="80%">[3:74.06714pt]</text></p></para>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p4""#],
    r##"<para xml:id="p4"><picture height="1" width="1" xml:id="p4.pic1"><svg:svg height="1" overflow="visible" version="1.1" viewBox="0 0 1 1" width="1"><svg:g transform="translate(0,1) matrix(1 0 0 -1 0 0)"/></svg:svg></picture><p><text font="italic" fontsize="144%">[4:24.79999pt]</text></p></para>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p5""#],
    r##"<para xml:id="p5"><picture height="1" width="1" xml:id="p5.pic1"><svg:svg height="1" overflow="visible" version="1.1" viewBox="0 0 1 1" width="1"><svg:g transform="translate(0,1) matrix(1 0 0 -1 0 0)"/></svg:svg></picture><p><text fontsize="80%">[5:17.9447]</text></p></para>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p6""#],
    r##"<para xml:id="p6"><picture height="8.39" width="50.21" xml:id="p6.pic1"><svg:svg height="8.39" overflow="visible" version="1.1" viewBox="0 0 50.21 8.39" width="50.21"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,8.39) matrix(1 0 0 -1 0 0) translate(25.11,0) translate(0,4.2)"><svg:path d="M -24.83 -3.92 h 49.66 v 7.84 h -49.66 Z" style="fill:none"/></svg:g></svg:svg></picture><p><text fontsize="80%">[6:36.2894pt]</text></p></para>"##,
  );
}
