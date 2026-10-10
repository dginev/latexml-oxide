//! Red/green guards for perfect-kernel batch 64w: a native binding of axodraw v1 (1994), the Feynman-diagram package
//! neither TeX Live nor most papers ship (run 336: 1,507 papers with `\Text` undefined), drawing its PostScript
//! procedures into the picture model.
use latexml::util::test::assert_element;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  svg_nested_picture::post_html,
};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// `\ArrowLine`, `\Photon`, `\Gluon`, `\Vertex` and `\Text` (hep-th0506235, hep-ph0512261, hep-ph0305218,
/// 0710.3255): the v1 `/arrowline` (a line, its arrowhead at the middle), `/photon` (`2·4 - 0.5` rounded = 8 half
/// wiggles, one `curveto` each), `/gluon` (head, 4 body windings, tail: 12 `curveto`s) and `/vertex` geometry, each
/// inside its `\put(\axoxoff,\axoyoff)`; the labels are `\makebox(0,0)[r]`/`[l]` at their `\unitlength` point.
/// Repro graphics-axodraw/axodraw_v1_feynman_vertex.
#[test]
fn axodraw_v1_feynman_vertex() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw_v1_feynman_vertex.tex"
  ));
  assert_element(
    &xml,
    "picture",
    &[],
    r##"<picture fill="none" height="100.0pt" origin-x="0.0pt" origin-y="0.0pt" stroke="none" unitlength="1.0pt" width="200.0pt" xml:id="p1.pic1">
    <g transform="translate(0,0)">
      <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)">
        <line points="13.84,69.19 83.02,69.19" stroke="#000000" stroke-width="0.69"/>
        <polygon fill="#000000" points="51.54,69.19 45.32,71.68 45.32,66.7" stroke="none"/>
      </g>
      <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)">
        <bezier points="83.02,69.19 83.85,76.13 85.15,76.92 91.67,74.37" stroke="#000000" stroke-width="0.69"/>
        <bezier points="91.67,74.37 98.19,71.83 99.5,72.62 100.32,79.56" stroke="#000000" stroke-width="0.69"/>
        <bezier points="100.32,79.56 101.14,86.51 102.45,87.3 108.97,84.75" stroke="#000000" stroke-width="0.69"/>
        <bezier points="108.97,84.75 115.49,82.21 116.79,82.99 117.62,89.94" stroke="#000000" stroke-width="0.69"/>
        <bezier points="117.62,89.94 118.44,96.89 119.75,97.67 126.26,95.13" stroke="#000000" stroke-width="0.69"/>
        <bezier points="126.26,95.13 132.78,92.59 134.09,93.37 134.91,100.32" stroke="#000000" stroke-width="0.69"/>
        <bezier points="134.91,100.32 135.74,107.27 137.04,108.05 143.56,105.51" stroke="#000000" stroke-width="0.69"/>
        <bezier points="143.56,105.51 150.08,102.96 151.39,103.75 152.21,110.7" stroke="#000000" stroke-width="0.69"/>
      </g>
      <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)">
        <bezier points="83.02,69.19 85.02,71.21 88.75,72.2 93.94,69.09" stroke="#000000" stroke-width="0.69"/>
        <bezier points="93.94,69.09 101.67,64.45 95.98,54.96 91.71,57.52" stroke="#000000" stroke-width="0.69"/>
        <bezier points="91.71,57.52 87.43,60.08 93.13,69.58 103.17,63.55" stroke="#000000" stroke-width="0.69"/>
        <bezier points="103.17,63.55 113.2,57.53 107.51,48.04 103.24,50.6" stroke="#000000" stroke-width="0.69"/>
        <bezier points="103.24,50.6 98.97,53.17 104.66,62.66 114.7,56.64" stroke="#000000" stroke-width="0.69"/>
        <bezier points="114.7,56.64 124.74,50.61 119.04,41.12 114.77,43.68" stroke="#000000" stroke-width="0.69"/>
        <bezier points="114.77,43.68 110.5,46.25 116.19,55.74 126.23,49.72" stroke="#000000" stroke-width="0.69"/>
        <bezier points="126.23,49.72 136.27,43.69 130.57,34.2 126.3,36.77" stroke="#000000" stroke-width="0.69"/>
        <bezier points="126.3,36.77 122.03,39.33 127.72,48.82 137.76,42.8" stroke="#000000" stroke-width="0.69"/>
        <bezier points="137.76,42.8 147.8,36.78 142.1,27.28 137.83,29.85" stroke="#000000" stroke-width="0.69"/>
        <bezier points="137.83,29.85 133.56,32.41 139.25,41.9 146.98,37.26" stroke="#000000" stroke-width="0.69"/>
        <bezier points="146.98,37.26 152.17,34.15 153.06,30.39 152.21,27.67" stroke="#000000" stroke-width="0.69"/>
      </g>
      <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)">
        <circle fill="#000000" r="2.77" stroke="none" x="83.02" y="69.19"/>
      </g>
      <g innerdepth="0.0pt" innerheight="6.6pt" innerwidth="7.9pt" transform="translate(6.92,69.19)">
        <g class="makebox" innerdepth="0.0pt" innerheight="6.6pt" innerwidth="7.9pt" transform="translate(-10.86,-4.6)">
          <Math mode="inline" tex="e^{-}" text="e ^ -" xml:id="p1.pic1.m1">
            <XMath xml:id="p1.pic1.m1.1">
              <XMApp xml:id="p1.pic1.m1.1.1">
                <XMTok role="SUPERSCRIPTOP" scriptpos="post1"/>
                <XMTok font="italic" role="UNKNOWN">e</XMTok>
                <XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok>
              </XMApp>
            </XMath>
          </Math>
        </g>
      </g>
      <g innerdepth="1.9pt" innerheight="4.3pt" innerwidth="5.2pt" transform="translate(159.13,110.7)">
        <g class="makebox" innerdepth="1.9pt" innerheight="4.3pt" innerwidth="5.2pt" transform="translate(0,-4.32)">
          <Math mode="inline" tex="\gamma" text="gamma" xml:id="p1.pic1.m2">
            <XMath xml:id="p1.pic1.m2.1">
              <XMTok font="italic" name="gamma" role="UNKNOWN">γ</XMTok>
            </XMath>
          </Math>
        </g>
      </g>
    </g>
  </picture>"##,
  );
}

/// `\CArc` over 360 degrees is the full circle; `\ArrowArc` is the anticlockwise arc with its `/arrowup` head at the
/// middle (90 degrees, pointing left); `\DashCArc` fits `[3 3]` to the arc's length, 2n+1 dashes (v1 `/dashcarc`);
/// `\GCirc`/`\GBox` fill with the gray level under a black outline; `\Oval(x,y)(h,w)(angle)` is the rotated ellipse
/// of semi-axes w and h. Repro
/// graphics-axodraw/axodraw_v1_arcs_boxes_ovals.
#[test]
fn axodraw_v1_arcs_boxes_ovals() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw_v1_arcs_boxes_ovals.tex"
  ));
  for (tag, attrs, expected) in [
    (
      "circle",
      ["x=\"41.51\"", "y=\"96.86\""],
      r##"<circle fill="none" r="27.67" stroke="#000000" stroke-width="1.38" x="41.51" y="96.86"/>"##,
    ),
    (
      "arc",
      ["angle2=\"180\"", "x=\"124.53\""],
      r##"<arc angle1="0" angle2="180" r="27.67" stroke="#000000" stroke-width="1.38" x="124.53" y="96.86"/>"##,
    ),
    (
      "polygon",
      ["points=\"120.38,", "fill=\"#000000\""],
      r##"<polygon fill="#000000" points="120.38,124.53 128.69,127.86 128.69,121.21" stroke="none"/>"##,
    ),
    (
      "arc",
      ["angle2=\"270\"", "x=\"207.56\""],
      r##"<arc angle1="0" angle2="270" r="27.67" stroke="#000000" stroke-dasharray="4.21 4.21" stroke-width="1.38" x="207.56" y="96.86"/>"##,
    ),
    (
      "circle",
      ["x=\"41.51\"", "y=\"27.67\""],
      r##"<circle fill="#b3b3b3" r="20.76" stroke="#000000" stroke-width="0.69" x="41.51" y="27.67"/>"##,
    ),
    (
      "rect",
      ["fill=\"#808080\"", "x=\"83.02\""],
      r##"<rect fill="#808080" height="41.51" stroke="#000000" stroke-width="0.69" width="55.35" x="83.02" y="6.92"/>"##,
    ),
    (
      "ellipse",
      ["x=\"207.56\"", "y=\"27.67\""],
      r##"<ellipse fill="none" rx="34.59" ry="13.84" stroke="#000000" stroke-width="0.69" transform="rotate(30,207.56,27.67)" x="207.56" y="27.67"/>"##,
    ),
    (
      "line",
      ["stroke=\"#000000\"", "points=\"276.74"],
      r##"<line points="276.74,13.84 387.44,124.53" stroke="#000000" stroke-width="0.69"/>"##,
    ),
  ] {
    assert_element(&xml, tag, &attrs, expected);
  }
  // The SVG keeps each arc's stroke, as SVG.pm's `convertArc` copies it (L385-387): it had dropped it, leaving the
  // arc to the picture's `stroke="none"`.
  let html = post_html(&xml);
  assert_element(
    &html,
    "path",
    &["d=\"M 152.20 96.86 A 27.67"],
    r##"<path d="M 152.20 96.86 A 27.67 27.67 0 0 1 96.86 96.86" stroke="#000000" stroke-width="1.38" style="--ltx-stroke-color:#000000;"></path>"##,
  );
}

/// 0809.0800's `\unitlength 0.6pt\SetScale{0.6}`: the graphics are in points scaled by `\SetScale`, ignoring
/// `\unitlength`; `\Text` is at `\unitlength`s, unscaled — the line from (0,0) to (100,100) ends at 60pt (83.02px),
/// where the label at (100,100) of 0.6pt units is. Repro graphics-axodraw/axodraw_v1_scale_and_unitlength.
#[test]
fn axodraw_v1_scale_and_unitlength() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw_v1_scale_and_unitlength.tex"
  ));
  assert_element(
    &xml,
    "line",
    &["points=\""],
    r##"<line points="0,0 83.02,83.02" stroke="#000000" stroke-width="0.42"/>"##,
  );
  assert_element(
    &xml,
    "g",
    &["transform=\"translate(83.02,83.02)\""],
    r##"<g innerdepth="0.0pt" innerheight="6.8pt" innerwidth="9.1pt" transform="translate(83.02,83.02)">
    <g class="makebox" innerdepth="0.0pt" innerheight="6.8pt" innerwidth="9.1pt" transform="translate(0,0)">
      <Math mode="inline" tex="X" text="X" xml:id="p1.pic1.m1">
        <XMath xml:id="p1.pic1.m1.1">
          <XMTok font="italic" role="UNKNOWN">X</XMTok>
        </XMath>
      </Math>
    </g>
  </g>"##,
  );
}

/// hep-ph0612301 (`\documentclass[dvips]{article}` with color): color's `dvips` option loads dvipsnam.def without
/// `usenames`, so `Blue` is known only as color's `named` model — still `\SetColor`'s color. Repro
/// graphics-axodraw/axodraw_v1_setcolor_dvips_option.
#[test]
fn axodraw_v1_setcolor_dvips_option() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw_v1_setcolor_dvips_option.tex"
  ));
  assert_element(
    &xml,
    "line",
    &["points=\""],
    r##"<line points="0,0 69.19,69.19" stroke="#0000FF" stroke-width="0.69"/>"##,
  );
}

/// 0709.2086, 1712.05312: `\SetColor{Purple}` with no color package names a colordvi color, as the colordvi-aware v1
/// `\input`s colordvi.sty: every curve of the gluon is dvipsnam's Purple. Repro
/// graphics-axodraw/axodraw_v1_setcolor_without_color_package.
#[test]
fn axodraw_v1_setcolor_without_color_package() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw_v1_setcolor_without_color_package.tex"
  ));
  assert_element(
    &xml,
    "picture",
    &[],
    r##"<picture fill="none" height="40.0pt" origin-x="0.0pt" origin-y="0.0pt" stroke="none" unitlength="1.0pt" width="100.0pt" xml:id="p1.pic1">
      <g transform="translate(0,0)">
        <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)">
          <bezier points="0,27.67 1.38,31.13 6.92,34.59 19.37,34.59" stroke="#8C24FF" stroke-width="0.69"/>
          <bezier points="19.37,34.59 33.9,34.59 33.9,20.76 27.67,20.76" stroke="#8C24FF" stroke-width="0.69"/>
          <bezier points="27.67,20.76 21.45,20.76 21.45,34.59 41.51,34.59" stroke="#8C24FF" stroke-width="0.69"/>
          <bezier points="41.51,34.59 61.58,34.59 61.58,20.76 55.35,20.76" stroke="#8C24FF" stroke-width="0.69"/>
          <bezier points="55.35,20.76 49.12,20.76 49.12,34.59 69.19,34.59" stroke="#8C24FF" stroke-width="0.69"/>
          <bezier points="69.19,34.59 89.25,34.59 89.25,20.76 83.02,20.76" stroke="#8C24FF" stroke-width="0.69"/>
          <bezier points="83.02,20.76 76.8,20.76 76.8,34.59 96.86,34.59" stroke="#8C24FF" stroke-width="0.69"/>
          <bezier points="96.86,34.59 116.92,34.59 116.92,20.76 110.7,20.76" stroke="#8C24FF" stroke-width="0.69"/>
          <bezier points="110.7,20.76 104.47,20.76 104.47,34.59 119,34.59" stroke="#8C24FF" stroke-width="0.69"/>
          <bezier points="119,34.59 131.45,34.59 136.99,31.13 138.37,27.67" stroke="#8C24FF" stroke-width="0.69"/>
        </g>
      </g>
    </picture>"##,
  );
}

/// 1701.01554, 0901.1195: v1 splices its arguments into PostScript, so `{20 Tdensity mul}` is an expression over the
/// constants `\special{! /Tdensity {0.5} def /Tamp {3}def}` defines (spaced and compact, recorded by `\special`): a
/// photon of amplitude 1 (`Tamp 2 sub`) and 10 wiggles. Repro graphics-axodraw/axodraw_v1_postscript_arguments.
#[test]
fn axodraw_v1_postscript_arguments() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw_v1_postscript_arguments.tex"
  ));
  assert_element(
    &xml,
    "picture",
    &[],
    r##"<picture fill="none" height="20.0pt" origin-x="0.0pt" origin-y="-10.0pt" stroke="none" unitlength="1.0pt" width="100.0pt" xml:id="p1.pic1">
      <g transform="translate(0,13.84)">
        <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)">
          <bezier points="0,0 2.94,1.84 3.98,1.84 6.92,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="6.92,0 9.85,-1.84 10.9,-1.84 13.84,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="13.84,0 16.77,1.84 17.82,1.84 20.76,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="20.76,0 23.69,-1.84 24.74,-1.84 27.67,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="27.67,0 30.61,1.84 31.66,1.84 34.59,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="34.59,0 37.53,-1.84 38.58,-1.84 41.51,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="41.51,0 44.45,1.84 45.49,1.84 48.43,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="48.43,0 51.37,-1.84 52.41,-1.84 55.35,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="55.35,0 58.29,1.84 59.33,1.84 62.27,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="62.27,0 65.2,-1.84 66.25,-1.84 69.19,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="69.19,0 72.12,1.84 73.17,1.84 76.1,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="76.1,0 79.04,-1.84 80.09,-1.84 83.02,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="83.02,0 85.96,1.84 87.01,1.84 89.94,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="89.94,0 92.88,-1.84 93.92,-1.84 96.86,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="96.86,0 99.8,1.84 100.84,1.84 103.78,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="103.78,0 106.72,-1.84 107.76,-1.84 110.7,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="110.7,0 113.63,1.84 114.68,1.84 117.62,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="117.62,0 120.55,-1.84 121.6,-1.84 124.53,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="124.53,0 127.47,1.84 128.52,1.84 131.45,0" stroke="#000000" stroke-width="0.69"/>
          <bezier points="131.45,0 134.39,-1.84 135.44,-1.84 138.37,0" stroke="#000000" stroke-width="0.69"/>
        </g>
      </g>
    </picture>"##,
  );
}

/// 1701.01404: `\usepackage{axodraw2}` is TeX Live's axodraw2.sty, raw-loaded as before v1 had a binding — the
/// versioned-name fallback must not answer it with the v1 binding (`{axopicture}`, `\Line[arrow]` are axodraw2's). Its
/// two warnings ask for `axohelp`, as pdflatex's do. Repro graphics-axodraw/axodraw2_is_not_v1.
#[test]
fn axodraw2_is_not_v1() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!("../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw2_is_not_v1.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 2, "{log}");
  assert!(!log.contains("falling back to axodraw.sty"), "{log}");
  assert!(!log.contains("axodraw_sty.rs"), "{log}");
  assert_element(
    &xml,
    "Math",
    &["tex=\"q\""],
    r##"<Math mode="inline" tex="q" text="q" xml:id="p1.pic1.pic1.m1">
      <XMath xml:id="p1.pic1.pic1.m1.1">
        <XMTok font="italic" role="UNKNOWN">q</XMTok>
      </XMath>
    </Math>"##,
  );
}

/// 0901.2111, 1101.4089, 1301.5845, 1701.08134 ship axodraw4j.sty (JaxoDraw's keyval forerunner of axodraw2): the
/// paper's own file is read, not the v1 binding. Repro graphics-axodraw/axodraw4j_shipped_copy_is_read (with its
/// stand-in axodraw4j.sty).
#[test]
fn axodraw4j_shipped_copy_is_read() {
  let (log, xml) = latexml::util::test::convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw4j_shipped_copy_is_read.tex"
    ),
    &[(
      "axodraw4j.sty",
      include_str!("../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw4j.sty"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(&xml, "p", &[], r##"<p xml:id="p1.1">axodraw4j read</p>"##);
}

/// pstricks' `\psarc`, the other producer of `ltx:arc`: its SVG path keeps the arc's stroke and `fill="none"`, as
/// SVG.pm's `convertArc` copies them (L385-387); the port emitted the `d` alone, an arc invisible under the picture's
/// `stroke="none"`. Repro graphics-tikz/pstricks_psarc_keeps_its_stroke.
#[test]
fn pstricks_psarc_keeps_its_stroke() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pstricks_psarc_keeps_its_stroke.tex"
  ));
  let html = post_html(&xml);
  assert_element(
    &html,
    "path",
    &["d=\"M 39.37 0.00 A 39.37"],
    r##"<path d="M 39.37 0.00 A 39.37 39.37 0 0 1 0.00 39.37" fill="none" stroke="black" stroke-width="0.8" style="--ltx-stroke-color:black;"></path>"##,
  );
}

/// 64w review: a diagram in math (`\[ A = \parbox{40pt}{\begin{picture}…\Photon…} + B \]`, 9 of 64 sampled 2008
/// axodraw papers) keeps its `\put` in the Math `tex` (the HTML `alttext`) and not the internal `\lx@axo@draw`: it has
/// no reversion, as the `\special` it stands for has no source to show. Repro
/// graphics-axodraw/axodraw_v1_diagram_in_math_tex_attribute.
#[test]
fn axodraw_v1_diagram_in_math_tex_attribute() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw_v1_diagram_in_math_tex_attribute.tex"
  ));
  let start = xml.find("<Math ").expect("a Math");
  let end = start + xml[start..].find('>').expect("its start tag");
  assert_eq!(
    &xml[start..=end],
    r##"<Math mode="display" tex="A=\parbox{40.0pt}{\begin{picture}(40.0,20.0)\put(0.0,0.0){}\end{picture}}+B" text="A = [] + B" xml:id="S0.Ex1.m1">"##
  );
}

/// 1301.5498: the colordvi-aware v1 `\input`s colordvi.sty, so its `\Black{$Z$}` is defined. Repro
/// graphics-axodraw/axodraw_v1_colordvi_text_command.
#[test]
fn axodraw_v1_colordvi_text_command() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw_v1_colordvi_text_command.tex"
  ));
  assert_element(
    &xml,
    "Math",
    &["tex=\"Z\""],
    r##"<Math mode="inline" tex="Z" text="Z" xml:id="p1.m1">
      <XMath xml:id="p1.m1.1">
        <XMTok font="italic" role="UNKNOWN">Z</XMTok>
      </XMath>
    </Math>"##,
  );
}

/// 64w review: loading colordvi keeps color's `\DefineNamedColor`, so a document's own named color resolves in
/// color's `named` model. Repro graphics-axodraw/axodraw_v1_keeps_color_definenamedcolor.
#[test]
fn axodraw_v1_keeps_color_definenamedcolor() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw_v1_keeps_color_definenamedcolor.tex"
  ));
  assert_element(
    &xml,
    "text",
    &["color=\"#0000FF\""],
    r##"<text color="#0000FF" xml:id="p1.1.1"> blue text</text>"##,
  );
}

/// 64w review: an infinite or huge winding count, an infinite or unbounded axis (`perdec` 0: an infinite step;
/// `perdec` -1: 200,003 marks), a NaN radius or angle, and an infinite line width — PostScript errors under dvips —
/// are each refused with one warning instead of running unbounded (a kill or an out-of-memory with no log) or drawing
/// NaN; the unreadable `\SetWidth` leaves the pen at 0.5pt, and the line after them is the picture's only drawing.
/// Repro graphics-axodraw/axodraw_v1_unbounded_arguments.
#[test]
fn axodraw_v1_unbounded_arguments() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-axodraw/axodraw_v1_unbounded_arguments.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 8, "{log}");
  assert_eq!(log.matches("Cannot draw axodraw").count(), 7, "{log}");
  assert_eq!(
    log
      .matches("Cannot read \\SetWidth value '1 0 div'")
      .count(),
    1,
    "{log}"
  );
  assert_element(
    &xml,
    "picture",
    &[],
    r##"<picture fill="none" height="40.0pt" origin-x="0.0pt" origin-y="0.0pt" stroke="none" unitlength="1.0pt" width="100.0pt" xml:id="p1.pic1">
      <g transform="translate(0,0)">
        <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)"/>
        <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)"/>
        <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)"/>
        <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)"/>
        <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)"/>
        <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)"/>
        <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)"/>
        <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)">
          <line points="0,0 138.37,0" stroke="#000000" stroke-width="0.69"/>
        </g>
      </g>
    </picture>"##,
  );
}

/// colordvi's read of dvipsnam.def no longer uses the file up: `\usepackage[dvips]{color}` after axodraw (which loads
/// colordvi) still reads it under color's `\DefineNamedColor`, so the `named` model knows Blue and Red. Repro
/// graphics-axodraw/colordvi_then_color_named_model.
#[test]
fn colordvi_then_color_named_model() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/colordvi_then_color_named_model.tex"
  ));
  assert_element(
    &xml,
    "text",
    &["color=\"#0000FF\""],
    r##"<text color="#0000FF" xml:id="p1.1.1"> blue text</text>"##,
  );
  assert_element(
    &xml,
    "text",
    &["color=\"#FF0000\""],
    r##"<text color="#FF0000" xml:id="p1.1.2"> red text</text>"##,
  );
  assert_element(
    &xml,
    "line",
    &[],
    r##"<line points="0,0 55.35,27.67" stroke="#FF0000" stroke-width="0.69"/>"##,
  );
}

/// color first, then axodraw's colordvi: color's on-demand read of dvipsnam.def for `\color[named]{Blue}` still
/// happens. Repro graphics-axodraw/color_then_colordvi_named_model.
#[test]
fn color_then_colordvi_named_model() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/color_then_colordvi_named_model.tex"
  ));
  assert_element(
    &xml,
    "text",
    &["color=\"#0000FF\""],
    r##"<text color="#0000FF" xml:id="p1.1.1">blue text</text>"##,
  );
}

/// `\usepackage[dvips]{color}` read dvipsnam.def first; colordvi still reads it for its own `\Red`/`\textRed` (TeX's
/// colordvi defines them from its own table), and color's `named` model keeps Blue. Repro
/// graphics-axodraw/color_dvips_then_colordvi_text_commands.
#[test]
fn color_dvips_then_colordvi_text_commands() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/color_dvips_then_colordvi_text_commands.tex"
  ));
  // The whole paragraph: `\Red`'s text is kept but still uncoloured (RED
  // `fonts-nfss/colordvi_named_text_color`); fixing that must update this pin.
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r##"<p xml:id="p1.1">Plain <text color="#0000FF" xml:id="p1.1.1"> blue text</text> and red text here.
<picture fill="none" height="20.0pt" origin-x="0.0pt" origin-y="0.0pt" stroke="none" unitlength="1.0pt" width="40.0pt" xml:id="p1.pic1">
        <g transform="translate(0,0)">
          <g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)">
            <line points="0,0 55.35,27.67" stroke="#FF0000" stroke-width="0.69"/>
          </g>
        </g>
      </picture></p>"##,
  );
}
