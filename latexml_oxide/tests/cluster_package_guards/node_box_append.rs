//! An auto-opened `ltx:picture` is sized from its node's box (tag
//! `afterClose`, latex_constructs.pool.ltxml:4943-4950), and that box is
//! Perl's whole record of what went into it: `openElementAt` appends the box
//! of every element opened inside an auto-opened node to it and to each
//! auto-opened ancestor (`appendNodeBox`, Core/Document.pm:1685-1700, called
//! at :1861-1862), and the drawing objects take box-free coordinate arguments
//! (`Pair`, `{Float}`, :4992-5078; pstricks_support.sty.ltxml:879-891), so they
//! measure nothing. Every expected size is same-host Perl `latexmlc`'s.
//! Repros `tools/perfect_kernel/repros/graphics-tikz/picture_autoopen_*.tex`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// Every `<picture …>` open tag of `xml`, in document order.
fn picture_open_tags(xml: &str) -> Vec<&str> { open_tags(xml, &["<picture"]) }

/// The `tex` attribute of every `<Math>` in `xml`, in document order.
fn math_tex_values(xml: &str) -> Vec<&str> {
  xml
    .match_indices("<Math ")
    .filter_map(|(i, _)| {
      let open = &xml[i..i + xml[i..].find('>').unwrap_or(0)];
      let start = open.find(" tex=\"")? + 6;
      Some(&open[start..start + open[start..].find('"')?])
    })
    .collect()
}

/// Every open tag of `xml` starting with one of `starts`, in document order.
fn open_tags<'a>(xml: &'a str, starts: &[&str]) -> Vec<&'a str> {
  let mut tags: Vec<(usize, &str)> = starts
    .iter()
    .flat_map(|start| {
      xml
        .match_indices(start)
        .map(|(i, _)| (i, &xml[i..=i + xml[i..].find('>').unwrap_or(0)]))
    })
    .collect();
  tags.sort_unstable_by_key(|(i, _)| *i);
  tags.into_iter().map(|(_, tag)| tag).collect()
}

/// Convert `tex` cleanly (0 errors, 0 warnings) and return its XML.
fn convert_clean(tex: &str) -> String {
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  xml
}

/// `\line Pair:Number {Float}` measures nothing: the picture is the " y"
/// written into it (Perl 11.92×8.65 px; the port that digested `(1,0){250}`
/// as text measured 57.27×11.61).
#[test]
fn line_takes_box_free_arguments() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/picture_autoopen_line_box_free.tex"
  ));
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p1.pic1""#],
    r##"<picture height="8.65" width="11.92" xml:id="p1.pic1"><line points="0,0 345.93,0" stroke="#000000" stroke-width="0.4"/><text>y</text></picture>"##,
  );
}

/// `\rput` opens its `<ltx:g>` in a whatsit with no box argument and writes
/// the body through `\put@end`, so the body is not measured (Perl 11.92×8.65
/// px, was 43.05×12.3); a `[refpoint]` is Perl's digested `[]`, its `pos`, and
/// is measured (Perl 23.45×12.3).
#[test]
fn rput_places_without_measuring_its_body() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/picture_autoopen_rput_box_free.tex"
  ));
  assert_eq!(
    picture_open_tags(&xml),
    [
      r#"<picture height="8.65" width="11.92" xml:id="p1.pic1">"#,
      r#"<picture height="12.3" width="23.45" xml:id="p2.pic1">"#,
    ],
    "{xml}"
  );
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p2.pic1""#],
    r#"<picture height="12.3" width="23.45" xml:id="p2.pic1"><g pos="bl" transform="translate(39.37,39.37)"><text>Hello</text></g><text>y</text></picture>"#,
  );
}

/// `\frame{X}`'s `\pic@makebox@` requests its content's width, height and
/// depth, which `getSize` returns as is: the picture is X plus " y" (Perl
/// 22.29×12.15 px; the `{framed=true}` and `[bl]` arguments measured as text
/// made it 111.16×12.3).
#[test]
fn frame_is_sized_as_its_content() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/picture_autoopen_frame_requested_size.tex"
  ));
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p1.pic1""#],
    r##"<picture height="12.15" width="22.29" xml:id="p1.pic1"><rect fill="none" height="6.83331pt" stroke="#000000" stroke-width="0.4" width="7.50002pt" x="0" y="0"/><g class="makebox" innerdepth="0.0pt" innerheight="6.83331pt" innerwidth="7.50002pt" transform="translate(0,0)"><text>X</text></g><text>y</text></picture>"##,
  );
}

/// A second `\put`, and a picture auto-opened inside the first picture's
/// auto-opened `ltx:text` — also inside a `\parbox`'s auto-opened `ltx:p` —
/// reach the outer picture's size, as Perl's `appendNodeBox` makes them.
#[test]
fn auto_opened_ancestors_measure_what_opens_inside() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/picture_autoopen_nested_append_node_box.tex"
  ));
  assert_eq!(
    picture_open_tags(&xml),
    [
      r#"<picture height="12.3" width="79.99" xml:id="p1.pic1">"#,
      r#"<picture height="12.3" width="43.24" xml:id="p2.pic1">"#,
      r#"<picture height="12.15" width="21.52" xml:id="p2.pic1.pic1">"#,
      r#"<picture height="12.3" width="38.63" xml:id="pic1">"#,
      r#"<picture height="12.15" width="16.91" xml:id="pic1.pic1">"#,
    ],
    "{xml}"
  );
}

/// Each LaTeX drawing object outside a picture auto-opens one sized as Perl
/// sizes it: nothing for the coordinates, the `*` of `\circle*`, the `[part]`
/// and requested size of `\oval`, the content of `\framebox`/`\dashbox`/
/// `\makebox`.
#[test]
fn drawing_objects_measure_no_coordinates() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/picture_autoopen_drawing_objects.tex"
  ));
  assert_eq!(
    picture_open_tags(&xml),
    [
      r#"<picture height="8.65" width="11.92" xml:id="p1.pic1">"#,
      r#"<picture height="8.65" width="11.92" xml:id="p2.pic1">"#,
      r#"<picture height="8.65" width="11.92" xml:id="p3.pic1">"#,
      r#"<picture height="13.07" width="18.83" xml:id="p4.pic1">"#,
      r#"<picture height="16.53" width="39.59" xml:id="p5.pic1">"#,
      r#"<picture height="8.65" width="11.92" xml:id="p6.pic1">"#,
      r#"<picture height="12.15" width="22.29" xml:id="p7.pic1">"#,
      r#"<picture height="12.15" width="20.95" xml:id="p8.pic1">"#,
      r#"<picture height="12.15" width="22.49" xml:id="p9.pic1">"#,
      r#"<picture height="12.15" width="24.6" xml:id="p10.pic1">"#,
    ],
    "{xml}"
  );
}

/// Replacing a node by others moves no box (Perl `replaceNode`,
/// Document.pm:2011-2025): the `\@framebox` unwrap inside an auto-opened
/// foreignObject or picture keeps the frame's padding and rule in their size.
/// Every size is Perl's but two pre-W12 widths: 23.53 (Perl 23.52) and the
/// `\parbox` foreignObject's 78.73 (Perl 88.15).
#[test]
fn replacing_a_node_keeps_the_boxes() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/picture_autoopen_replace_node_box_neutral.tex"
  ));
  assert_eq!(
    open_tags(&xml, &["<picture", "<svg:foreignObject"]),
    [
      r#"<picture height="29.9" width="32.75" xml:id="p1.pic1">"#,
      r#"<svg:foreignObject height="20.67" overflow="visible" style="--ltx-fo-width:1.7em;--ltx-fo-height:1.15em;--ltx-fo-depth:0.34em;font-size:10pt;" transform="matrix(1 0 0 -1 0 15.97)" width="23.53">"#,
      r#"<picture height="51.84" width="42.92" xml:id="p2.pic1">"#,
      r#"<svg:foreignObject height="42.62" overflow="visible" style="--ltx-fo-width:2.44em;--ltx-fo-height:1.79em;--ltx-fo-depth:1.29em;font-size:10pt;" transform="matrix(1 0 0 -1 0 24.77)" width="33.7">"#,
      r#"<picture height="28.8" width="97.92" xml:id="p3.pic1">"#,
      r#"<svg:foreignObject height="19.02" overflow="visible" style="--ltx-fo-width:5.69em;--ltx-fo-height:1.03em;--ltx-fo-depth:0.34em;font-size:10pt;" transform="matrix(1 0 0 -1 0 14.31)" width="78.73">"#,
      r#"<picture height="35.47" width="61.68" xml:id="p4.pic1">"#,
    ],
    "{xml}"
  );
}

/// A Math that unwraps to text only (`cleanup_math`, Perl `replaceTree`)
/// counts once: its box leaves the auto-opened parent, and only a
/// foreignObject left with no box takes it back (Perl makes that one an
/// `svg:text`, which Rust does not, SYNC_STATUS). Every size is Perl's; the
/// last foreignObject is Perl's `svg:text`.
#[test]
fn math_text_pieces_are_counted_once() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/picture_autoopen_math_text_pieces.tex"
  ));
  assert_eq!(
    open_tags(&xml, &["<picture", "<svg:foreignObject"]),
    [
      r#"<picture height="12.3" width="36.9" xml:id="p1.pic1">"#,
      r#"<picture height="12.3" width="36.9" xml:id="p2.pic1">"#,
      r#"<picture height="12.15" width="22.29" xml:id="p3.pic1">"#,
      r#"<picture height="12.3" width="50.74" xml:id="p4.pic1">"#,
      r#"<picture height="18.83" width="23.83" xml:id="p5.pic1">"#,
      r#"<svg:foreignObject height="9.61" overflow="visible" style="--ltx-fo-width:1.06em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="14.61">"#,
      r#"<picture height="18.83" width="37.67" xml:id="p6.pic1">"#,
      r#"<svg:foreignObject height="9.61" overflow="visible" style="--ltx-fo-width:1.06em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="14.61">"#,
      r#"<picture height="9.22" width="23.06" xml:id="p7.pic1">"#,
      r#"<svg:foreignObject height="0" overflow="visible" style="--ltx-fo-width:1em;--ltx-fo-height:0em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 0)" width="13.84">"#,
    ],
    "{xml}"
  );
  // The text pieces themselves: each appears once, in order (Perl's content).
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p2.pic1""#],
    r#"<picture height="12.3" width="36.9" xml:id="p2.pic1"><g innerdepth="0.0pt" innerheight="6.8pt" innerwidth="7.5pt" transform="translate(0,0)"><text>A</text></g><text class="ltx_markedasmath">ab</text><text>y</text></picture>"#,
  );
}

/// `\bezier{N}` is stroked (Perl's template has no `stroke`, KNOWN_PERL_ERRORS
/// #271) and carries `displayedpoints` for a nonzero count only, as Perl's
/// string test does: `\qbezier[0]` draws solid.
#[test]
fn bezier_is_stroked_and_counts_its_points() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/picture_bezier_stroke_and_count.tex"
  ));
  assert_eq!(
    open_tags(&xml, &["<picture", "<bezier"]),
    [
      r#"<picture fill="none" height="50.0pt" stroke="none" unitlength="1.0pt" width="100.0pt" xml:id="p1.pic1">"#,
      r##"<bezier displayedpoints="20" points="0,0 69.19,69.19 138.37,0" stroke="#000000" stroke-width="0.4"/>"##,
      r#"<picture fill="none" height="50.0pt" stroke="none" unitlength="1.0pt" width="100.0pt" xml:id="p2.pic1">"#,
      r##"<bezier points="0,0 69.19,69.19 138.37,0" stroke="#000000" stroke-width="0.4"/>"##,
      r#"<picture height="8.65" width="11.92" xml:id="p3.pic1">"#,
      r##"<bezier displayedpoints="20" points="0,0 69.19,69.19 138.37,0" stroke="#000000" stroke-width="0.4"/>"##,
      r#"<picture fill="none" height="50.0pt" stroke="none" unitlength="1.0pt" width="100.0pt" xml:id="p4.pic1">"#,
      r##"<bezier points="0,0 69.19,69.19 138.37,0" stroke="#000000" stroke-width="0.4"/>"##,
    ],
    "{xml}"
  );
}

/// `\line` without `(` is plain TeX's `\hbox to\hsize` (OXIDIZED_DESIGN_DIVERGENCES
/// #316; witness 2306.13101), where Perl reads a picture object: three errors and
/// a Fatal (`slopeToPicCoord`), and the conversion fails; `\line(…)` stays the
/// picture object.
#[test]
fn line_outside_a_picture_is_a_full_width_box() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/picture_line_outside_picture.tex"
  ));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para class="ltx_noindent" xml:id="p1"><p><text width="345.0pt">Left Right</text></p></para>"#,
  );
  assert_eq!(
    picture_open_tags(&xml),
    [r#"<picture height="8.65" width="11.92" xml:id="p2.pic1">"#],
    "{xml}"
  );
}

/// A math ligature (`2 . 414` → `2.414`) keeps the merged tokens' boxes in
/// the boxes they were part of: an aligned cell Math's `tex=` is complete,
/// where Perl's `removeNodeBox` drops the `2.` and the `1` of `10`
/// (KNOWN_PERL_ERRORS #272; witnesses 2605.02288, 2605.00812).
#[test]
fn math_ligatures_keep_their_boxes() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/node_box_math_ligature_tex.tex"
  ));
  assert_eq!(
    math_tex_values(&xml),
    [
      r"\displaystyle\text{Long}=2.414\times 10^{-3}",
      r"\displaystyle\text{Long}",
      r"\displaystyle=2.414\times 10^{-3}",
      r"\displaystyle\text{Height}=2.351\times 10^{-3}",
      r"\displaystyle\text{Height}",
      r"\displaystyle=2.351\times 10^{-3}",
      r"\hyperref@@ii[defn:cu]{\mathsf{CUA}}_{\mathrm{U}}",
      "CUA",
    ],
    "{xml}"
  );
}

/// A standalone panel (a subfigure grid's caption line of two
/// `0.49\textwidth` boxes) is a row of its own, and the next row starts empty:
/// four `0.24\textwidth` panels after it share one row, as in pdflatex, where
/// Perl carries the line's width into that row and splits it 1 | 3
/// (KNOWN_PERL_ERRORS #274; witness 2605.02317).
#[test]
fn a_standalone_panel_row_starts_the_next_row_empty() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/node_box_panel_row_after_caption_line.tex"
  ));
  let panel = |n: usize| {
    format!(
      r#"<figure align="center" class="ltx_figure_panel" placement="b" xml:id="S0.F1.fig{n}"><p>P</p></figure>"#
    )
  };
  let line = r#"<p align="center" class="ltx_figure_panel"><text align="center" fontsize="90%" width="169.1pt">Left</text><text fontsize="90%"> <text align="center" width="169.1pt">Right</text></text></p>"#;
  let brk = r#"<break class="ltx_break"/>"#;
  let expected = format!(
    r#"<figure inlist="lof" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags>{}{brk}{line}{brk}{}{brk}{line}<toccaption class="ltx_centering"><tag close=" ">1</tag>C</toccaption><caption class="ltx_centering"><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">C</text></caption></figure>"#,
    (1..=4).map(panel).collect::<String>(),
    (5..=8).map(panel).collect::<String>(),
  );
  assert_element(&xml, "figure", &[r#"xml:id="S0.F1""#], &expected);
}

/// A node whose content lives on keeps its box in place: an XMText renamed to
/// XMWrap (`cleanup_XMText`) keeps a `\mbox`, `\raisebox`, `\hbox`,
/// `\resizebox` or `\scalebox` in the aligned cell's `tex`, where Perl drops
/// the first three (KNOWN_PERL_ERRORS #272), and `\sideset`'s copied nucleus
/// keeps the `\sideset` (Perl moves it). Every value is 56jk's.
#[test]
fn aligned_cells_keep_their_boxed_pieces() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/node_box_aligned_boxed_pieces.tex"
  ));
  let rows = [
    ("a", r"\sideset{{}_{a}}{{}^{b}}{\sum}_{i}x_{i}"),
    ("h", r"\mbox{$x$}+\text{{ab}}+\textbf{c}"),
    ("i", r"\resizebox{3729359}{}{$x+y$}+\raisebox{1.0pt}{$z$}"),
    ("l", r"\hbox{$x$}+\vbox{\hbox{$y$}}+\phantom{x}+\smash{y}"),
    ("h", r"\scalebox{0.95}{$x+y$}"),
    ("h", r"\scalebox{0.95}{$x$}"),
    ("h", r"\scalebox{0.95}{$x+y$}"),
  ];
  let expected: Vec<String> = rows
    .iter()
    .flat_map(|(lhs, rhs)| {
      [
        format!(r"\displaystyle {lhs}={rhs}"),
        format!(r"\displaystyle {lhs}"),
        format!(r"\displaystyle={rhs}"),
      ]
    })
    .collect();
  assert_eq!(math_tex_values(&xml), expected, "{xml}");
}

/// A small panel merged into the block after it goes first in the block,
/// keeping source order (Perl appends it last, KNOWN_PERL_ERRORS #274): the
/// 5pt image precedes the minipage's or parbox's content.
#[test]
fn a_panel_merged_into_a_block_keeps_its_place() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/node_box_panel_merge_keeps_order.tex"
  ));
  let line = r#"<p class="ltx_figure_panel">A long line of text that spans most of the line width in the figure, filling it up.</p><break class="ltx_break"/>"#;
  let figure = |n: usize, lead: &str, class: &str, body: &str, cap: &str| {
    format!(
      r#"<figure inlist="lof" xml:id="S0.F{n}"><tags><tag>Figure {n}</tag><tag role="refnum">{n}</tag><tag role="typerefnum">Figure {n}</tag></tags>{lead}<block class="ltx_figure_panel {class}" vattach="middle" width="172.5pt"><graphics class="ltx_figure_panel" graphic="none.png" options="width=5.0pt,keepaspectratio=true" xml:id="S0.F{n}.g1"/>{body}</block><toccaption><tag close=" ">{n}</tag>{cap}</toccaption><caption><tag close=": ">Figure {n}</tag>{cap}</caption></figure>"#
    )
  };
  let one_two = "<p>ONE</p><p>TWO</p>";
  for (n, expected) in (1..).zip([
    figure(
      1,
      line,
      "ltx_minipage",
      r#"<tabular vattach="middle"><tbody><tr><td align="center">T1</td></tr></tbody></tabular><p>MINI</p>"#,
      "C",
    ),
    figure(2, line, "ltx_parbox", one_two, "E"),
    figure(3, "", "ltx_minipage", one_two, "D"),
  ]) {
    let id = format!(r#"xml:id="S0.F{n}""#);
    assert_element(&xml, "figure", &[&id], &expected);
  }
}
