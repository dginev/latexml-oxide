//! Guards for HALIGNINT: text leaving a pgf picture through `\pgfinterruptpicture` (a node, its label or pin, an edge
//! node, a node part, `\pgftext`) sets `\halign` as TeX does, while tikz's `align=` lines and a matrix in a picture stay
//! the SVG matrix (`\lxSVG@uninstallcommands`, pgfsys_latexml_def.rs; OXIDIZED_DESIGN_DIVERGENCES #1167). A new module:
//! no table of a picture's `\halign` readings exists. The nested `\ooalign` shape is
//! `perfect_kernel_batchpanic397::a_nested_ooalign_in_node_math_reverts`.

use latexml::util::test::{
  assert_element, check_requirements, convert_with, error_count, warning_count,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/",
      $name,
      ".tex"
    ))
  };
}

/// Rows: the repro, the packages it needs (skipped visibly where one is absent), the number of SVG matrices
/// (`class="ltx_tikzmatrix"`: a tikz-cd diagram, a `matrix of nodes`, an `align=` node's lines), then each (tag,
/// start-tag fragments, the whole element). Every conversion 0 E / 0 W (pdflatex 0 errors). On the base the node text's
/// `\halign` was the SVG matrix: 1 Error in the tikz-cd label (1901.06584, 2409.06632, 2409.04903, 2103.05078), 4 in
/// the tableaux (1502.07025, 2604.27868), and the formula or text emptied silently elsewhere (Perl: 1, `\ydiagram`
/// unbound, 1 and 6 Errors, KNOWN_PERL_ERRORS #1340). The last row is byte-identical on both binaries.
#[test]
fn text_leaving_a_picture_sets_halign_as_tex() {
  type Row = (
    &'static str,
    &'static str,
    &'static [&'static str],
    usize,
    &'static [(&'static str, &'static [&'static str], &'static str)],
  );
  const ROWS: &[Row] = &[
    (
      "ooalign_in_a_tikzcd_label_is_an_alignment",
      repro!("ooalign_in_a_tikzcd_label_is_an_alignment"),
      &["tikz-cd.sty"],
      1,
      &[(
        "Math",
        &[r#"xml:id="p1.pic1.m5""#],
        r#"
            <Math mode="inline" tex="\scriptstyle{\mathrel{\vtop{\halign{#\cr$\scriptstyle\rightarrow$\cr$\scriptstyle\mid$\cr}}}}" text="Array[[rightarrow], [mid]]" xml:id="p1.pic1.m5">
            <XMath xml:id="p1.pic1.m5.1">
            <XMArray role="RELOP" vattach="top" xml:id="p1.pic1.m5.1.1">
            <XMRow xml:id="p1.pic1.m5.1.1.1">
            <XMCell align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.m5.1.1.1.1">
            <XMTok fontsize="49%" name="rightarrow" role="ARROW">→</XMTok>
            </XMCell>
            </XMRow>
            <XMRow xml:id="p1.pic1.m5.1.1.2">
            <XMCell align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.m5.1.1.2.1">
            <XMTok fontsize="49%" name="mid" role="VERTBAR">∣</XMTok>
            </XMCell>
            </XMRow>
            </XMArray>
            </XMath>
            </Math>
          "#,
      )],
    ),
    (
      "ydiagram_in_a_tikzcd_cell_and_a_math_matrix",
      repro!("ydiagram_in_a_tikzcd_cell_and_a_math_matrix"),
      &["tikz-cd.sty", "ytableau.sty"],
      2,
      &[
        (
          "tabular",
          &[r#"xml:id="p2.pic1.1.1""#],
          r#"
            <tabular vattach="top" xml:id="p2.pic1.1.1">
            <tr xml:id="p2.pic1.1.1.1">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p2.pic1.1.1.1.1"><text yoffset="-0.4pt" xml:id="p2.pic1.1.1.1.1.1"><inline-block vattach="bottom" xml:id="p2.pic1.1.1.1.1.1.1">
            <p xml:id="p2.pic1.1.1.1.1.1.1.1"><inline-block vattach="bottom" xml:id="p2.pic1.1.1.1.1.1.1.1.1">
            <p width="15.0pt" xml:id="p2.pic1.1.1.1.1.1.1.1.1.1"/>
            </inline-block></p>
            <rule height="0.4pt" width="15.8pt"/>
            <rule height="0.4pt" width="15.8pt"/>
            </inline-block></text></td>
            </tr>
            <tr xml:id="p2.pic1.1.1.2">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p2.pic1.1.1.2.1"><text yoffset="-0.4pt" xml:id="p2.pic1.1.1.2.1.1"><inline-block vattach="bottom" xml:id="p2.pic1.1.1.2.1.1.1">
            <p xml:id="p2.pic1.1.1.2.1.1.1.1"><inline-block vattach="bottom" xml:id="p2.pic1.1.1.2.1.1.1.1.1">
            <p width="15.0pt" xml:id="p2.pic1.1.1.2.1.1.1.1.1.1"/>
            </inline-block></p>
            <rule height="0.4pt" width="15.8pt"/>
            <rule height="0.4pt" width="15.8pt"/>
            </inline-block></text></td>
            </tr>
            </tabular>
          "#,
        ),
        (
          "Math",
          &[r#"xml:id="p1.pic1.m2""#],
          r#"
            <Math mode="inline" tex="{B}" text="B" xml:id="p1.pic1.m2">
            <XMath xml:id="p1.pic1.m2.1">
            <XMTok font="italic" role="UNKNOWN">B</XMTok>
            </XMath>
            </Math>
          "#,
        ),
      ],
    ),
    (
      "halign_in_node_math_is_an_array",
      repro!("halign_in_node_math_is_an_array"),
      &[],
      0,
      &[(
        "Math",
        &[r#"xml:id="p1.pic1.m1""#],
        r#"
            <Math mode="inline" tex="x_{\vtop{\halign{#\cr a\cr b\cr}}}" text="x _ Array[[[a]], [[b]]]" xml:id="p1.pic1.m1">
            <XMath xml:id="p1.pic1.m1.1">
            <XMApp xml:id="p1.pic1.m1.1.1">
            <XMTok role="SUBSCRIPTOP" scriptpos="post1"/>
            <XMTok font="italic" role="UNKNOWN">x</XMTok>
            <XMArray vattach="top" xml:id="p1.pic1.m1.1.1.3">
            <XMRow xml:id="p1.pic1.m1.1.1.3.1">
            <XMCell align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.m1.1.1.3.1.1">
            <XMText xml:id="p1.pic1.m1.1.1.3.1.1.1"><text fontsize="70%" xml:id="p1.pic1.m1.1.1.3.1.1.1.1">a</text></XMText>
            </XMCell>
            </XMRow>
            <XMRow xml:id="p1.pic1.m1.1.1.3.2">
            <XMCell align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.m1.1.1.3.2.1">
            <XMText xml:id="p1.pic1.m1.1.1.3.2.1.1"><text fontsize="70%" xml:id="p1.pic1.m1.1.1.3.2.1.1.1">b</text></XMText>
            </XMCell>
            </XMRow>
            </XMArray>
            </XMApp>
            </XMath>
            </Math>
          "#,
      )],
    ),
    (
      "halign_on_every_route_out_of_a_picture",
      repro!("halign_on_every_route_out_of_a_picture"),
      &[],
      0,
      &[
        (
          "text",
          &[r#"xml:id="p1.pic1.1""#],
          r#"
            <text xml:id="p1.pic1.1">T<tabular vattach="top" xml:id="p1.pic1.1.1">
            <tr xml:id="p1.pic1.1.1.1">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.1.1.1.1">a</td>
            </tr>
            <tr xml:id="p1.pic1.1.1.2">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.1.1.2.1">b</td>
            </tr>
            </tabular></text>
          "#,
        ),
        (
          "text",
          &[r#"xml:id="p1.pic1.3""#],
          r#"
            <text xml:id="p1.pic1.3">L<tabular vattach="top" xml:id="p1.pic1.3.1">
            <tr xml:id="p1.pic1.3.1.1">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.3.1.1.1">a</td>
            </tr>
            <tr xml:id="p1.pic1.3.1.2">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.3.1.2.1">b</td>
            </tr>
            </tabular></text>
          "#,
        ),
        (
          "text",
          &[r#"xml:id="p1.pic1.4""#],
          r#"
            <text xml:id="p1.pic1.4">P<tabular vattach="top" xml:id="p1.pic1.4.1">
            <tr xml:id="p1.pic1.4.1.1">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.4.1.1.1">a</td>
            </tr>
            <tr xml:id="p1.pic1.4.1.2">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.4.1.2.1">b</td>
            </tr>
            </tabular></text>
          "#,
        ),
        (
          "text",
          &[r#"xml:id="p1.pic1.5""#],
          r#"
            <text xml:id="p1.pic1.5">E<tabular vattach="top" xml:id="p1.pic1.5.1">
            <tr xml:id="p1.pic1.5.1.1">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.5.1.1.1">a</td>
            </tr>
            <tr xml:id="p1.pic1.5.1.2">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.5.1.2.1">b</td>
            </tr>
            </tabular></text>
          "#,
        ),
        (
          "text",
          &[r#"xml:id="p1.pic1.6""#],
          r#"
            <text xml:id="p1.pic1.6">U<tabular vattach="top" xml:id="p1.pic1.6.1">
            <tr xml:id="p1.pic1.6.1.1">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.6.1.1.1">a</td>
            </tr>
            <tr xml:id="p1.pic1.6.1.2">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.6.1.2.1">b</td>
            </tr>
            </tabular></text>
          "#,
        ),
        (
          "text",
          &[r#"xml:id="p1.pic1.7""#],
          r#"
            <text xml:id="p1.pic1.7">V<tabular vattach="top" xml:id="p1.pic1.7.1">
            <tr xml:id="p1.pic1.7.1.1">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.7.1.1.1">a</td>
            </tr>
            <tr xml:id="p1.pic1.7.1.2">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.7.1.2.1">b</td>
            </tr>
            </tabular></text>
          "#,
        ),
      ],
    ),
    (
      "align_lines_and_nested_matrices_stay_svg",
      repro!("align_lines_and_nested_matrices_stay_svg"),
      &[],
      4,
      &[
        (
          "svg:g",
          &[
            r#"class="ltx_tikzmatrix""#,
            r#"transform="matrix(1 0 0 -1 0 21.76)""#,
          ],
          r#"
            <svg:g class="ltx_tikzmatrix" transform="matrix(1 0 0 -1 0 21.76)">
            <svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 9.46)">
            <svg:g class="ltx_tikzmatrix_col ltx_nopad_r" transform="matrix(1 0 0 -1 0 0)">
            <svg:foreignObject height="12.15" overflow="visible" style="--ltx-fo-width:1.78em;--ltx-fo-height:0.68em;--ltx-fo-depth:0.19em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.46)" width="24.6">Top</svg:foreignObject>
            </svg:g>
            </svg:g>
            <svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 21.76)">
            <svg:g class="ltx_tikzmatrix_col ltx_nopad_r" transform="matrix(1 0 0 -1 0.19 0)">
            <svg:foreignObject height="9.61" overflow="visible" style="--ltx-fo-width:1.67em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="23.06">mid</svg:foreignObject>
            </svg:g>
            </svg:g>
            </svg:g>
          "#,
        ),
        (
          "svg:g",
          &[
            r#"class="ltx_tikzmatrix""#,
            r#"transform="matrix(1 0 0 -1 0 28.44)""#,
          ],
          r##"
            <svg:g class="ltx_tikzmatrix" transform="matrix(1 0 0 -1 0 28.44)">
            <svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 10.57)">
            <svg:g class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r" fill="#000000" stroke="#000000" transform="matrix(1 0 0 -1 0 0) translate(10.76,0) matrix(1.0 0.0 0.0 1.0 -6.15 0)">
            <svg:foreignObject height="8.65" overflow="visible" style="--ltx-fo-width:0.89em;--ltx-fo-height:0.43em;--ltx-fo-depth:0.19em;font-size:10pt;" transform="matrix(1 0 0 -1 0 5.96)" width="12.3"><text xml:id="p1.pic1.pic1.1">p </text></svg:foreignObject>
            </svg:g>
            <svg:g class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r" fill="#000000" stroke="#000000" transform="matrix(1 0 0 -1 21.52 0) translate(8.26,0) matrix(1.0 0.0 0.0 1.0 -3.65 0)">
            <svg:foreignObject height="8.65" overflow="visible" style="--ltx-fo-width:0.53em;--ltx-fo-height:0.43em;--ltx-fo-depth:0.19em;font-size:10pt;" transform="matrix(1 0 0 -1 0 5.96)" width="7.3"><text xml:id="p1.pic1.pic1.2">q</text></svg:foreignObject>
            </svg:g>
            </svg:g>
            <svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 28.44)">
            <svg:g class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r" fill="#000000" stroke="#000000" transform="matrix(1 0 0 -1 1.13 0) translate(9.63,0) matrix(1.0 0.0 0.0 1.0 -5.02 0)">
            <svg:foreignObject height="5.96" overflow="visible" style="--ltx-fo-width:0.73em;--ltx-fo-height:0.43em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 5.96)" width="10.03"><text xml:id="p1.pic1.pic1.3">r </text></svg:foreignObject>
            </svg:g>
            <svg:g class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r" fill="#000000" stroke="#000000" transform="matrix(1 0 0 -1 22.45 0) translate(7.34,0) matrix(1.0 0.0 0.0 1.0 -2.73 0)">
            <svg:foreignObject height="5.96" overflow="visible" style="--ltx-fo-width:0.39em;--ltx-fo-height:0.43em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 5.96)" width="5.46"><text xml:id="p1.pic1.pic1.4">s</text></svg:foreignObject>
            </svg:g>
            </svg:g>
            </svg:g>
          "##,
        ),
        (
          "Math",
          &[r#"xml:id="p1.pic1.m1""#],
          r#"
            <Math mode="inline" tex="\begin{pmatrix}a&amp;b\\&#10;c&amp;d\end{pmatrix}" text="matrix@(Array[[a, b], [c, d]])" xml:id="p1.pic1.m1">
            <XMath xml:id="p1.pic1.m1.2">
            <XMDual xml:id="p1.pic1.m1.2.1">
            <XMApp xml:id="p1.pic1.m1.2.1.1">
            <XMTok meaning="matrix"/>
            <XMRef idref="p1.pic1.m1.1" xml:id="p1.pic1.m1.2.1.1.2"/>
            </XMApp>
            <XMWrap xml:id="p1.pic1.m1.2.1.2">
            <XMTok role="OPEN" stretchy="true">(</XMTok>
            <XMArray xml:id="p1.pic1.m1.1">
            <XMRow xml:id="p1.pic1.m1.1.1">
            <XMCell align="center" xml:id="p1.pic1.m1.1.1.1">
            <XMTok font="italic" role="UNKNOWN">a</XMTok>
            </XMCell>
            <XMCell align="center" xml:id="p1.pic1.m1.1.1.2">
            <XMTok font="italic" role="UNKNOWN">b</XMTok>
            </XMCell>
            </XMRow>
            <XMRow xml:id="p1.pic1.m1.1.2">
            <XMCell align="center" xml:id="p1.pic1.m1.1.2.1">
            <XMTok font="italic" role="UNKNOWN">c</XMTok>
            </XMCell>
            <XMCell align="center" xml:id="p1.pic1.m1.1.2.2">
            <XMTok font="italic" role="UNKNOWN">d</XMTok>
            </XMCell>
            </XMRow>
            </XMArray>
            <XMTok role="CLOSE" stretchy="true">)</XMTok>
            </XMWrap>
            </XMDual>
            </XMath>
            </Math>
          "#,
        ),
        (
          "Math",
          &[r#"xml:id="p1.pic1.m2""#],
          r#"
            <Math mode="inline" tex="f=\begin{cases}1&amp;x&gt;0\\&#10;0&amp;\text{else}\end{cases}" text="f = cases@(1, x &gt; 0, 0, [else])" xml:id="p1.pic1.m2">
            <XMath xml:id="p1.pic1.m2.5">
            <XMApp xml:id="p1.pic1.m2.5.1">
            <XMTok meaning="equals" role="RELOP">=</XMTok>
            <XMTok font="italic" role="UNKNOWN">f</XMTok>
            <XMDual xml:id="p1.pic1.m2.5.1.3">
            <XMApp xml:id="p1.pic1.m2.5.1.3.1">
            <XMTok meaning="cases"/>
            <XMRef idref="p1.pic1.m2.1" xml:id="p1.pic1.m2.5.1.3.1.2"/>
            <XMRef idref="p1.pic1.m2.2" xml:id="p1.pic1.m2.5.1.3.1.3"/>
            <XMRef idref="p1.pic1.m2.3" xml:id="p1.pic1.m2.5.1.3.1.4"/>
            <XMRef idref="p1.pic1.m2.4" xml:id="p1.pic1.m2.5.1.3.1.5"/>
            </XMApp>
            <XMWrap xml:id="p1.pic1.m2.5.1.3.2">
            <XMTok role="OPEN" stretchy="true">{</XMTok>
            <XMArray xml:id="p1.pic1.m2.5.1.3.2.2">
            <XMRow xml:id="p1.pic1.m2.5.1.3.2.2.1">
            <XMCell align="left" xml:id="p1.pic1.m2.5.1.3.2.2.1.1">
            <XMTok meaning="1" role="NUMBER" xml:id="p1.pic1.m2.1">1</XMTok>
            </XMCell>
            <XMCell align="left" xml:id="p1.pic1.m2.5.1.3.2.2.1.2">
            <XMApp xml:id="p1.pic1.m2.2">
            <XMTok meaning="greater-than" role="RELOP">&gt;</XMTok>
            <XMTok font="italic" role="UNKNOWN">x</XMTok>
            <XMTok meaning="0" role="NUMBER">0</XMTok>
            </XMApp>
            </XMCell>
            </XMRow>
            <XMRow xml:id="p1.pic1.m2.5.1.3.2.2.2">
            <XMCell align="left" xml:id="p1.pic1.m2.5.1.3.2.2.2.1">
            <XMTok meaning="0" role="NUMBER" xml:id="p1.pic1.m2.3">0</XMTok>
            </XMCell>
            <XMCell align="left" xml:id="p1.pic1.m2.5.1.3.2.2.2.2">
            <XMText xml:id="p1.pic1.m2.4">else</XMText>
            </XMCell>
            </XMRow>
            </XMArray>
            </XMWrap>
            </XMDual>
            </XMApp>
            </XMath>
            </Math>
          "#,
        ),
      ],
    ),
  ];
  for (name, tex, needs, matrices, elements) in ROWS {
    if !check_requirements(needs) {
      continue;
    }
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert_eq!(
      xml.matches(r#"class="ltx_tikzmatrix""#).count(),
      *matrices,
      "{name}: {xml}"
    );
    for (tag, attrs, expected) in *elements {
      assert_element(&xml, tag, attrs, expected);
    }
  }
}
