//! The TeX box family — `\box`, `\copy`, `\lastbox`, `\vsplit`, `\unhbox`, `\unvbox`, `\unhcopy`, `\unvcopy` and box
//! sizes — as one collection (user 2026-10-02): each repro in tools/perfect_kernel/repros/boxes-groups/box_primitives_*
//! holds one case per paragraph, checked against pdflatex, its header naming every case's status and root (design
//! agent_reports/2026-10-02_box_family_design.md, R1-R6). The tests assert the GREEN cases as whole elements; a RED
//! case joins when its root is fixed.

use super::perfect_kernel_batch57::{RAW, assert_elements, assert_elements_with};

fn case(id: &'static str, markup: &'static str) -> (&'static str, &'static str, &'static str) {
  ("para", id, markup)
}

/// `\lastbox` (tex.web §1080) and the list-tail registers (§424).
#[test]
fn lastbox() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/box_primitives_lastbox.tex"),
    RAW,
    (0, 0),
    &[
      case(
        "p1",
        r##"<para xml:id="p1"><p>Kept line.Next paragraph.</p></para>"##,
      ),
      case(
        "p2",
        r##"<para xml:id="p2"><p>Also kept.</p><p>Inner.</p></para>"##,
      ),
      case("p3", r##"<para xml:id="p3"><p>[47.28606pt]</p></para>"##),
      case("p4", r##"<para xml:id="p4"><p>[b/a]</p></para>"##),
      case("p8", r##"<para xml:id="p8"><p>[1]</p></para>"##),
      case("p9", r##"<para xml:id="p9"><p>[0]</p></para>"##),
      case("p10", r##"<para xml:id="p10"><p>[0.0pt]</p></para>"##),
      case("p11", r##"<para xml:id="p11"><p>[0.0pt] done.</p></para>"##),
      case("p12", r##"<para xml:id="p12"><p>[h]</p></para>"##),
      case("p13", r##"<para xml:id="p13"><p>[void]</p></para>"##),
      case("p14", r##"<para xml:id="p14"><p>[3.0pt]</p></para>"##),
      case("p15", r##"<para xml:id="p15"><p>[void]</p></para>"##),
      case("p16", r##"<para xml:id="p16"><p>[void]</p></para>"##),
      case("p18", r##"<para xml:id="p18"><p>[3.0pt]</p></para>"##),
      case("p19", r##"<para xml:id="p19"><p>[0.0pt]</p></para>"##),
      case("p20", r##"<para xml:id="p20"><p>[other]</p></para>"##),
      case("p21", r##"<para xml:id="p21"><p>[void/xy]</p></para>"##),
      case(
        "p22",
        r##"<para xml:id="p22"><p>[void/<Math mode="inline" tex="x" text="x" xml:id="p22.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math>]</p></para>"##,
      ),
      case("p23", r##"<para xml:id="p23"><p>[void]</p></para>"##),
      case("p24", r##"<para xml:id="p24"><p>[other]</p></para>"##),
      case(
        "p27",
        r##"<para xml:id="p27"><p>[other,5.2778pt]</p></para>"##,
      ),
      case("p28", r##"<para xml:id="p28"><p>[other]</p></para>"##),
      case("p29", r##"<para xml:id="p29"><p>[void]</p></para>"##),
      case(
        "p31",
        r##"<para xml:id="p31"><p>[6.83331pt,7.08336pt]</p></para>"##,
      ),
      case("p32", r##"<para xml:id="p32"><p>[void]</p></para>"##),
      case("p33", r##"<para xml:id="p33"><p>[box]Alpha.</p></para>"##),
      case(
        "p34",
        r##"<para class="ltx_noindent" xml:id="p34"><p>[void]Bravo.</p></para>"##,
      ),
    ],
  );
}

/// `\vsplit` (tex.web §968-977).
#[test]
fn vsplit() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/box_primitives_vsplit.tex"),
    RAW,
    (0, 0),
    &[
      case(
        "p3",
        r##"<para class="ltx_noindent" xml:id="p3"><p vattach="bottom">Three</p></para>"##,
      ),
      case("p4", r##"<para xml:id="p4"><p>[5.0pt,10.0pt]</p></para>"##),
      case(
        "p5",
        r##"<para vattach="top" xml:id="p5"><p>Alpha</p></para>"##,
      ),
      case(
        "p6",
        r##"<para xml:id="p6"><p>Beta</p><p>Gamma</p></para>"##,
      ),
      case("p7", r##"<para xml:id="p7"><p>[1]</p></para>"##),
      case(
        "p9",
        r##"<para xml:id="p9"><p>F1</p><p>F2</p><p>F3</p></para>"##,
      ),
      case(
        "p10",
        r##"<para vattach="bottom" xml:id="p10"><p>N3</p></para>"##,
      ),
      case(
        "p11",
        r##"<para xml:id="p11"><p>[20.0pt,5.0pt,10.0pt]</p></para>"##,
      ),
      case(
        "p12",
        r##"<para xml:id="p12"><p>[24.0pt,22.0pt]</p></para>"##,
      ),
      case(
        "p13",
        r##"<para xml:id="p13"><p>[15.0pt,22.0pt]</p></para>"##,
      ),
      case(
        "p14",
        r##"<para xml:id="p14"><p>[5.0pt,1.0pt,10.0pt]</p></para>"##,
      ),
      case(
        "p15",
        r##"<para xml:id="p15"><p>[10.0pt,5.0pt]</p></para>"##,
      ),
      case(
        "p16",
        r##"<para xml:id="p16"><p>[22.0pt,0.0pt]</p></para>"##,
      ),
      case(
        "p17",
        r##"<para xml:id="p17"><p>[10.0pt,10.0pt]</p></para>"##,
      ),
      case(
        "p21",
        r##"<para xml:id="p21"><p>[10.0pt,32.0pt]</p></para>"##,
      ),
      case("p22", r##"<para xml:id="p22"><p>[6.83331pt]</p></para>"##),
      case("p23", r##"<para xml:id="p23"><p>[6.83331pt]</p></para>"##),
      case("p25", r##"<para xml:id="p25"><p>[22.0pt]</p></para>"##),
      case("p26", r##"<para xml:id="p26"><p>[22.0pt]</p></para>"##),
      case("p29", r##"<para xml:id="p29"><p>[28.33344pt]</p></para>"##),
    ],
  );
}

/// The unpacking primitives (§1110) and box sizes (§1083, `\vtop` §1087).
#[test]
fn unpack() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/box_primitives_unpack.tex"),
    RAW,
    (0, 0),
    &[
      case("p2", r##"<para xml:id="p2"><p>[10.55559pt]</p></para>"##),
      case("p3", r##"<para xml:id="p3"><p>[ab]</p></para>"##),
      case("p5", r##"<para xml:id="p5"><p>[1]</p></para>"##),
      case(
        "p6",
        r##"<para xml:id="p6"><p>[50.0pt,6.83331pt,full]</p></para>"##,
      ),
      case(
        "p10",
        r##"<para xml:id="p10"><p>[6.83331pt,43.16669pt]</p></para>"##,
      ),
      case(
        "p11",
        r##"<para xml:id="p11"><p>[6.83331pt,23.94444pt]</p></para>"##,
      ),
      case(
        "p12",
        r##"<para xml:id="p12"><p>[28.83331pt,1.94444pt]</p></para>"##,
      ),
      case(
        "p13",
        r##"<para xml:id="p13"><p>[0.0pt,11.83331pt]</p></para>"##,
      ),
      case(
        "p14",
        r##"<para xml:id="p14"><p>[0.0pt,20.77776pt]</p></para>"##,
      ),
      case(
        "p17",
        r##"<para xml:id="p17"><p>[0.0pt,6.83331pt]</p></para>"##,
      ),
      case("p18", r##"<para xml:id="p18"><p>[12.0pt]</p></para>"##),
      case(
        "p19",
        r##"<para xml:id="p19"><p>[0.0pt,12.0pt]</p></para>"##,
      ),
      case("p20", r##"<para xml:id="p20"><p>[5.2778pt]</p></para>"##),
      case(
        "p23",
        r##"<para xml:id="p23"><p>[11.91666pt,6.91666pt]</p></para>"##,
      ),
      case(
        "p27",
        r##"<para xml:id="p27"><p><inline-block class="ltx_parbox" vattach="top" width="56.9pt"><p>x</p></inline-block><inline-block class="ltx_parbox" vattach="top" width="56.9pt"><p>y</p></inline-block>tail</p></para>"##,
      ),
      case(
        "p28",
        r##"<para class="ltx_noindent" xml:id="p28"><p><inline-block class="ltx_parbox" vattach="top" width="56.9pt"><p>x</p></inline-block>tail</p></para>"##,
      ),
      case(
        "p29",
        r##"<para xml:id="p29"><p>[10.55559pt/10.55559pt]</p></para>"##,
      ),
      case(
        "p34",
        r##"<para xml:id="p34"><p>x<inline-block angle="90" depth="0.0pt" height="56.9pt" innerdepth="1.9pt" innerheight="4.3pt" innerwidth="56.9pt" width="6.3pt" xtranslate="-25.3pt" ytranslate="-25.3pt"><p class="ltx_parbox" vattach="middle" width="56.9pt">y</p></inline-block>tail</p></para>"##,
      ),
      case(
        "p39",
        r##"<para xml:id="p39"><p>[345.0pt: 6.83331pt+24.0pt]</p></para>"##,
      ),
      case(
        "p41",
        r##"<para xml:id="p41"><p>[38.77776pt+0.0pt]</p></para>"##,
      ),
      case(
        "p38",
        r##"<para xml:id="p38"><p>x<inline-block angle="90" depth="0.0pt" height="56.9pt" innerdepth="1.9pt" innerheight="4.3pt" innerwidth="56.9pt" width="0.0pt" xtranslate="-25.3pt" ytranslate="-25.3pt"><p class="ltx_parbox" vattach="middle" width="56.9pt">y</p></inline-block>tail</p></para>"##,
      ),
    ],
  );
}

/// `\unhcopy` inside `$\emph{…}$`, which TeX typesets as text (`\emph` in math is `\nfss@text`; LaTeXML too since 62b,
/// OXIDIZED_DESIGN_DIVERGENCES #440): the box goes back whole, no error, its text kept as the emphasis (review r2 of 59y:
/// an "Incompatible list" error there lost the "m").
#[test]
fn unhcopy_in_math_text_keeps_the_box() {
  assert_elements(
    "\\documentclass{article}\\begin{document}\\setbox0\\hbox{m}$\\emph{\\unhcopy0}$\\end{document}",
    RAW,
    (0, 0),
    &[case(
      "p1",
      r##"<para xml:id="p1"><p><emph class="ltx_markedasmath">m</emph></p></para>"##,
    )],
  );
}

/// A TikZ node's `align=` lines are `\halign` cells holding the `\unhbox`ed line: a phantom opening a line is its
/// content, not column padding (2605.03603). Repro boxes-groups/tikz_align_line_padding (c4 RED).
#[test]
fn tikz_align_line_padding() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/tikz_align_line_padding.tex"),
    RAW,
    (0, 0),
    &[
      case(
        "p1",
        r##"<para xml:id="p1"><picture height="30.98" width="52.08" xml:id="p1.pic1"><svg:svg height="30.98" overflow="visible" version="1.1" viewBox="0 0 52.08 30.98" width="52.08"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,30.98) matrix(1 0 0 -1 0 0) translate(26.04,0) translate(0,15.49) matrix(1.0 0.0 0.0 1.0 -21.43 -10.88)"><svg:g class="ltx_tikzmatrix" transform="matrix(1 0 0 -1 0 21.76)"><svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 9.61)"><svg:g class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r" transform="matrix(1 0 0 -1 0 0)"><svg:foreignObject height="12.3" overflow="visible" style="--ltx-fo-width:2.64em;--ltx-fo-height:0.69em;--ltx-fo-depth:0.19em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="36.51">Alpha</svg:foreignObject></svg:g></svg:g><svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 21.76)"><svg:g class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r" transform="matrix(1 0 0 -1 0 0)"><svg:foreignObject height="9.46" overflow="visible" style="--ltx-fo-width:3.1em;--ltx-fo-height:0.68em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.46)" width="42.86"><text class="ltx_phantom">xx</text>Beta</svg:foreignObject></svg:g></svg:g></svg:g></svg:g></svg:svg></picture></para>"##,
      ),
      case(
        "p2",
        r##"<para xml:id="p2"><picture height="30.98" width="54.73" xml:id="p2.pic1"><svg:svg height="30.98" overflow="visible" version="1.1" viewBox="0 0 54.73 30.98" width="54.73"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,30.98) matrix(1 0 0 -1 0 0) translate(27.36,0) translate(0,15.49) matrix(1.0 0.0 0.0 1.0 -22.75 -10.88)"><svg:g class="ltx_tikzmatrix" transform="matrix(1 0 0 -1 0 21.76)"><svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 9.61)"><svg:g class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r" transform="matrix(1 0 0 -1 0 0)"><svg:foreignObject height="12.3" overflow="visible" style="--ltx-fo-width:2.64em;--ltx-fo-height:0.69em;--ltx-fo-depth:0.19em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="36.51">Alpha</svg:foreignObject></svg:g></svg:g><svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 21.76)"><svg:g class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r" transform="matrix(1 0 0 -1 0 0)"><svg:foreignObject height="11.53" overflow="visible" style="--ltx-fo-width:3.29em;--ltx-fo-height:0.68em;--ltx-fo-depth:0.15em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.46)" width="45.51"><text class="ltx_phantom"><Math mode="inline" tex="v_{1}:" text="v _ 1 colon absent" xml:id="p2.pic1.m1"><XMath><XMApp><XMTok name="colon" role="METARELOP">:</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">v</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok meaning="absent"/></XMApp></XMath></Math></text>Beta</svg:foreignObject></svg:g></svg:g></svg:g></svg:g></svg:svg></picture></para>"##,
      ),
      case(
        "p3",
        r##"<para xml:id="p3"><picture height="30.98" width="59.34" xml:id="p3.pic1"><svg:svg height="30.98" overflow="visible" version="1.1" viewBox="0 0 59.34 30.98" width="59.34"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,30.98) matrix(1 0 0 -1 0 0) translate(29.67,0) translate(0,15.49) matrix(1.0 0.0 0.0 1.0 -25.06 -10.88)"><svg:g class="ltx_tikzmatrix" transform="matrix(1 0 0 -1 0 21.76)"><svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 9.61)"><svg:g class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r" transform="matrix(1 0 0 -1 0 0)"><svg:foreignObject height="12.3" overflow="visible" style="--ltx-fo-width:2.64em;--ltx-fo-height:0.69em;--ltx-fo-depth:0.19em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="36.51">Alpha</svg:foreignObject></svg:g></svg:g><svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 21.76)"><svg:g class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r" transform="matrix(1 0 0 -1 0 0)"><svg:foreignObject height="11.53" overflow="visible" style="--ltx-fo-width:3.62em;--ltx-fo-height:0.68em;--ltx-fo-depth:0.15em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.46)" width="50.12"><text class="ltx_phantom"><Math mode="inline" tex="v_{1}:" text="v _ 1 colon absent" xml:id="p3.pic1.m1"><XMath><XMApp><XMTok name="colon" role="METARELOP">:</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">v</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok meaning="absent"/></XMApp></XMath></Math></text> Beta</svg:foreignObject></svg:g></svg:g></svg:g></svg:g></svg:svg></picture></para>"##,
      ),
    ],
  );
}

/// An alignment cell's leading phantom (with a width) is its content, not padding; zero-width phantoms are peeled as
/// before. Repro boxes-groups/alignment_cell_phantom (c2 RED).
#[test]
fn alignment_cell_phantom() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/alignment_cell_phantom.tex"),
    RAW,
    (0, 0),
    &[
      case(
        "p1",
        r##"<para xml:id="p1"><tabular vattach="middle"><tbody><tr><td align="left"><text class="ltx_phantom">0</text>5</td></tr><tr><td align="left">12</td></tr></tbody></tabular></para>"##,
      ),
      case(
        "p3",
        r##"<para xml:id="p3"><tabular class="ltx_guessed_headers" vattach="middle"><tbody><tr><td align="left" border="r" thead="row">5</td><td align="right">5</td></tr><tr><td align="left" border="r" thead="row">12</td><td align="right">12</td></tr><tr><td align="center" border="r" thead="row">x</td><td align="right">y</td></tr><tr><td align="left" border="r" thead="row">z</td><td align="right">w</td></tr></tbody></tabular></para>"##,
      ),
      case(
        "p4",
        r##"<para xml:id="p4"><tabular vattach="middle"><tbody><tr><td align="left">a</td></tr><tr><td align="left">b</td></tr></tbody></tabular></para>"##,
      ),
    ],
  );
}

/// `\unskip` removes horizontal glue — an interword space, `\ `, `\quad`, `\hfil`, `~`, `\hspace` — and leaves kerns
/// (tex.web §1105-1106; KNOWN_PERL_ERRORS #436). Repro boxes-groups/unskip_horizontal_glue (09, 15, 17, 19-21, 24 RED).
#[test]
fn unskip_horizontal_glue() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/unskip_horizontal_glue.tex"),
    RAW,
    (0, 0),
    &[
      case(
        "p1",
        r##"<para class="ltx_noindent" xml:id="p1"><p>01[a,]</p></para>"##,
      ),
      case(
        "p2",
        r##"<para class="ltx_noindent" xml:id="p2"><p>02[a,]</p></para>"##,
      ),
      case(
        "p3",
        r##"<para class="ltx_noindent" xml:id="p3"><p>03[a,]</p></para>"##,
      ),
      case(
        "p4",
        r##"<para class="ltx_noindent" xml:id="p4"><p>04[a,]</p></para>"##,
      ),
      case(
        "p5",
        r##"<para class="ltx_noindent" xml:id="p5"><p>05[a,]</p></para>"##,
      ),
      case(
        "p6",
        r##"<para class="ltx_noindent" xml:id="p6"><p>06[a,]</p></para>"##,
      ),
      case(
        "p7",
        r##"<para class="ltx_noindent" xml:id="p7"><p>07[a 1.66672pt]</p></para>"##,
      ),
      case(
        "p8",
        r##"<para class="ltx_noindent" xml:id="p8"><p>08[a 2.0pt]</p></para>"##,
      ),
      case(
        "p10",
        r##"<para class="ltx_noindent" xml:id="p10"><p>10[a,]</p></para>"##,
      ),
      case(
        "p11",
        r##"<para class="ltx_noindent" xml:id="p11"><p>11[<Math mode="inline" tex="a" text="a" xml:id="p11.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math>,]</p></para>"##,
      ),
      case(
        "p12",
        r##"<para class="ltx_noindent" xml:id="p12"><p>12[a,]</p></para>"##,
      ),
      case("p13", r##"<para xml:id="p13"><p>[13]</p></para>"##),
      case(
        "p14",
        r##"<para class="ltx_noindent" xml:id="p14"><p>14[a,]</p></para>"##,
      ),
      case(
        "p16",
        r##"<para class="ltx_noindent" xml:id="p16"><p>16[a 10.00002pt]</p></para>"##,
      ),
      case(
        "p18",
        r##"<para class="ltx_noindent" xml:id="p18"><p>18[a,]</p></para>"##,
      ),
      case(
        "p22",
        r##"<para class="ltx_noindent" xml:id="p22"><p>22[a,]</p></para>"##,
      ),
      case(
        "p23",
        r##"<para class="ltx_noindent" xml:id="p23"><p>23[a 5.0pt]</p></para>"##,
      ),
      case(
        "p25",
        r##"<para class="ltx_noindent" xml:id="p25"><p>25[a,]</p></para>"##,
      ),
      case(
        "p26",
        r##"<para class="ltx_noindent" xml:id="p26"><p>26[a 1.66672pt]</p></para>"##,
      ),
      case(
        "p27",
        r##"<para class="ltx_noindent" xml:id="p27"><p>27[a-1.66672pt]</p></para>"##,
      ),
      case(
        "p28",
        r##"<para class="ltx_noindent" xml:id="p28"><p>28[a 2.22198pt]</p></para>"##,
      ),
      case(
        "p29",
        r##"<para class="ltx_noindent" xml:id="p29"><p>29[a 2.22198pt]</p></para>"##,
      ),
      case(
        "p30",
        r##"<para class="ltx_noindent" xml:id="p30"><p>30[a 2.77695pt]</p></para>"##,
      ),
      case(
        "p31",
        "<para class=\"ltx_noindent\" xml:id=\"p31\"><p>31[a\u{200B}-1.66672pt]</p></para>",
      ),
      case(
        "p32",
        r##"<para class="ltx_noindent" xml:id="p32"><p>32[a b]</p></para>"##,
      ),
      case(
        "p33",
        r##"<para class="ltx_noindent" xml:id="p33"><p>33[a 3.33333pt]</p></para>"##,
      ),
      case(
        "p34",
        r##"<para class="ltx_noindent" xml:id="p34"><p>34[a 3.33333pt]</p></para>"##,
      ),
      case(
        "p35",
        r##"<para class="ltx_noindent" xml:id="p35"><p>35[<text fontsize="120%">a 3.91663pt</text>]</p></para>"##,
      ),
      case(
        "p36",
        r##"<para class="ltx_noindent" xml:id="p36"><p>36[a 3.33333pt]</p></para>"##,
      ),
    ],
  );
}

/// `\vadjust` material is a vertical list after its line, each `\vadjust` a group whose paragraph ends at the group's
/// end (two in one line are two paragraphs; a `\bfseries` in one stays in it); it survives a group around the
/// `\vadjust`; material that is no paragraph adds none; a restricted horizontal box (`\settowidth`, `\sbox`, `\mbox`)
/// keeps what it queued, printing nothing (calc's `\widthof` and pgfmath's `width()` too), and a `\par` in one ends
/// no paragraph; before a display it is set before the display (tex.web §655, §889, §1096-1100, §1199;
/// KNOWN_PERL_ERRORS #441). The queued `\lx@normal@par` cannot be told from `\par` here: under
/// LaTeX a material ending with `\par` = `\relax` loops in pdflatex (`\partokencontext`). Repro
/// boxes-groups/vadjust_material_is_a_vertical_list.
#[test]
fn vadjust() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/vadjust_material_is_a_vertical_list.tex"
    ),
    RAW,
    (0, 0),
    &[
      case(
        "p1",
        r##"<para class="ltx_noindent" xml:id="p1"><p>Alpha  Charlie.</p></para>"##,
      ),
      case(
        "p2",
        r##"<para class="ltx_noindent" xml:id="p2"><p>Bravo</p></para>"##,
      ),
      case(
        "p3",
        r##"<para class="ltx_noindent" xml:id="p3"><p>Delta  Foxtrot.</p></para>"##,
      ),
      case(
        "p4",
        r##"<para class="ltx_noindent" xml:id="p4"><p>Echo</p></para>"##,
      ),
      case(
        "p5",
        r##"<para class="ltx_noindent" xml:id="p5"><p>Hotel Kilo.</p></para>"##,
      ),
      case(
        "p6",
        r##"<para class="ltx_noindent" xml:id="p6"><p>India</p></para>"##,
      ),
      case(
        "p7",
        r##"<para class="ltx_noindent" xml:id="p7"><p>Juliet</p></para>"##,
      ),
      case(
        "p8",
        r##"<para class="ltx_noindent" xml:id="p8"><p>Lima November.</p></para>"##,
      ),
      case(
        "p9",
        r##"<para class="ltx_noindent" xml:id="p9"><p><text font="bold">Mike</text></p></para>"##,
      ),
      case(
        "p10",
        r##"<para class="ltx_noindent" xml:id="p10"><p>Oscar.</p></para>"##,
      ),
      case(
        "p11",
        r##"<para class="ltx_noindent" xml:id="p11"><p>Papa Quebec.</p></para>"##,
      ),
      case(
        "p12",
        r##"<para class="ltx_noindent" xml:id="p12"><p>Victor  Xray.</p></para>"##,
      ),
      case(
        "p13",
        r##"<para class="ltx_noindent" xml:id="p13"><p>Yankee</p><p>Zulu</p><equation xml:id="S0.Ex1"><Math mode="display" tex="x" text="x" xml:id="S0.Ex1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></equation><p>Omega.</p></para>"##,
      ),
      case(
        "p14",
        r##"<para class="ltx_noindent" xml:id="p14"><p>Amber CoralDune Ember.</p></para>"##,
      ),
      case(
        "p15",
        r##"<para class="ltx_noindent" xml:id="p15"><p>Beryl</p></para>"##,
      ),
      case(
        "p16",
        r##"<para class="ltx_noindent" xml:id="p16"><p>Jade.</p></para>"##,
      ),
      case(
        "p17",
        r##"<para class="ltx_noindent" xml:id="p17"><p>Lemon melon.</p></para>"##,
      ),
      case(
        "p18",
        r##"<para class="ltx_noindent" xml:id="p18"><p>Mango</p></para>"##,
      ),
      case(
        "p19",
        r##"<para class="ltx_noindent" xml:id="p19"><p>Nectar olive.</p></para>"##,
      ),
      case(
        "p20",
        r##"<para class="ltx_noindent" xml:id="p20"><p>Peach rhubarb.</p></para>"##,
      ),
      case(
        "p21",
        r##"<para class="ltx_noindent" xml:id="p21"><p>Quince</p></para>"##,
      ),
      case(
        "p22",
        r##"<para class="ltx_noindent" xml:id="p22"><p>Raspberry strawberry.</p></para>"##,
      ),
      case(
        "p23",
        r##"<para class="ltx_noindent" xml:id="p23"><p>Tangerine</p></para>"##,
      ),
    ],
  );
  assert!(
    !xml.contains(r#"xml:id="p24""#),
    "one paragraph too many:\n{xml}"
  );
  let at = |needle: &str| {
    xml
      .find(needle)
      .unwrap_or_else(|| panic!("no {needle}:\n{xml}"))
  };
  let page_break = at(r#"<pagination role="newpage"/>"#);
  assert!(
    at(r#"xml:id="p19""#) < page_break && page_break < at(r#"xml:id="p20""#),
    "\\pagebreak's page break is not after Nectar's paragraph:\n{xml}"
  );
  for dropped in [
    "Romeo", "Sierra", "Tango", "Uniform", "Whiskey", "Flint", "Garnet", "Haze", "Iris",
  ] {
    assert!(
      !xml.contains(dropped),
      "{dropped} is measured or boxed, never set:\n{xml}"
    );
  }
}

/// `\vadjust` reads its material live after its `{`, as the box loop reads a box's (tex.web §1099), so a short
/// verbatim in it prints its `\ifx` and the paragraph that follows is untouched; Perl and Rust read a macro argument
/// first, so the `\ifx` ran. Repro boxes-groups/vadjust_material_is_read_live; witness etextools-examples line 279.
#[test]
fn vadjust_material_is_read_live() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/vadjust_material_is_read_live.tex"
    ),
    RAW,
    (0, 0),
    &[
      case(
        "p1",
        r##"<para class="ltx_noindent" xml:id="p1"><p>Alpha Delta.</p></para>"##,
      ),
      case(
        "p2",
        r##"<para class="ltx_noindent" xml:id="p2"><p>Bravo <text font="typewriter">\ifx</text> charlie.</p></para>"##,
      ),
      case(
        "p3",
        r##"<para xml:id="p3"><p><text font="typewriter">Echo.</text></p></para>"##,
      ),
    ],
  );
}

/// `\vadjust` builds its material where it is read (tex.web §1099), so a box register, a macro and a counter are as
/// they are there, not at the paragraph's end. Repro boxes-groups/vadjust_material_is_built_where_it_is_read.
#[test]
fn vadjust_material_is_built_where_it_is_read() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/vadjust_material_is_built_where_it_is_read.tex"
    ),
    RAW,
    (0, 0),
    &[
      case(
        "p1",
        r##"<para class="ltx_noindent" xml:id="p1"><p>Alpha  Charlie.</p></para>"##,
      ),
      case("p2", r##"<para xml:id="p2"><p>Bravo</p></para>"##),
      case(
        "p3",
        r##"<para class="ltx_noindent" xml:id="p3"><p>Delta  Foxtrot.</p></para>"##,
      ),
      case(
        "p4",
        r##"<para class="ltx_noindent" xml:id="p4"><p>Echo</p></para>"##,
      ),
      case(
        "p5",
        r##"<para class="ltx_noindent" xml:id="p5"><p>Hotel Juliet.</p></para>"##,
      ),
      case(
        "p6",
        r##"<para class="ltx_noindent" xml:id="p6"><p>India 0</p></para>"##,
      ),
    ],
  );
}

/// A `\vadjust` whose material has no `{` reports "Missing { inserted" and takes the material to the next `}` (tex.web
/// §403 `scan_left_brace`, gullet.rs): `\vadjust x}` sets "x" after its line, as pdflatex after the same error.
#[test]
fn vadjust_missing_brace_is_inserted() {
  assert_elements_with(
    "\\documentclass{article}\\begin{document}\n\\noindent Alpha\\vadjust x} bravo.\\par\n\\end{document}\n",
    RAW,
    (1, 0),
    &["Missing { inserted"],
    &[
      case(
        "p1",
        r##"<para class="ltx_noindent" xml:id="p1"><p>Alpha bravo.</p></para>"##,
      ),
      case("p2", r##"<para xml:id="p2"><p>x</p></para>"##),
    ],
  );
}
