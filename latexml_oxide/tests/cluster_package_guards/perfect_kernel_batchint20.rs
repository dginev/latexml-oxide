//! Red/green guards for the int20 fix round (the 70a + 70b bundle A/B's worse rows): AMS-TeX's `\Let@` sets the row
//! end to `\cr` (amstex.tex:486), graphicx reads a graphic's name at the document's catcodes, a sub-file gincltex boxes
//! has its preamble skipped beside standalone, and `\backslashbox` reads its arguments inside the cell.
//!
//! Every repro is checked against pdflatex/pdftex (its header holds the oracle).
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::assert_elements,
};

/// The arXiv configuration's preload.
const AR5IV: &str = "ar5iv.sty";

/// amstex/amstex_let_row_end_is_cr: a document's AMS-TeX alignment macro calls `\Let@` before its `\halign`
/// (diag.tex's `\newCD`), so `\\` is `\cr` (amstex.tex:486), not amsmath's `\math@cr`, whose `\@ifstar` plain AMS-TeX
/// lacks (math9611222, math9611223, math9412232, math0005042).
#[test]
fn amstex_let_sets_the_row_end_to_cr() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/amstex/amstex_let_row_end_is_cr.tex"),
    AR5IV,
    (0, 0),
    &[("tabular", "id1.1.1", TABULAR_2X2)],
  );
}

const TABULAR_2X2: &str = r##"<tabular class="ltx_markedasmath" vattach="bottom" xml:id="id1.1.1"><tr xml:id="id1.1.1.1"><td align="center" xml:id="id1.1.1.1.1"><Math mode="inline" tex="A" text="A" xml:id="id1.1.1.1.1.1"><XMath xml:id="id1.1.1.1.1.1.1"><XMTok font="italic" role="UNKNOWN">A</XMTok></XMath></Math></td><td align="center" xml:id="id1.1.1.1.2"><Math mode="inline" tex="B" text="B" xml:id="id1.1.1.1.2.1"><XMath xml:id="id1.1.1.1.2.1.1"><XMTok font="italic" role="UNKNOWN">B</XMTok></XMath></Math></td></tr><tr xml:id="id1.1.1.2"><td align="center" xml:id="id1.1.1.2.1"><Math mode="inline" tex="C" text="C" xml:id="id1.1.1.2.1.1"><XMath xml:id="id1.1.1.2.1.1.1"><XMTok font="italic" role="UNKNOWN">C</XMTok></XMath></Math></td><td align="center" xml:id="id1.1.1.2.2"><Math mode="inline" tex="D" text="D" xml:id="id1.1.1.2.2.1"><XMath xml:id="id1.1.1.2.2.1.1"><XMTok font="italic" role="UNKNOWN">D</XMTok></XMath></Math></td></tr></tabular>"##;

/// graphics-tikz/graphic_name_macro_with_underscore: graphicx reads the graphic's name at the document's catcodes and
/// expands it (graphicx.sty:198-203, `\Gin@setfile`), so `\def\seq_name{fig}`'s parameter text `_name` matches the
/// subscript `_` of `a-\seq_name-b` (1908.04391, 2609.16620).
#[test]
fn graphic_name_macro_with_underscore_expands() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/graphic_name_macro_with_underscore.tex"
    ),
    AR5IV,
    (0, 0),
    &[("para", "p1", GRAPHICS_PARA)],
  );
}

const GRAPHICS_PARA: &str = r##"<para xml:id="p1"><graphics graphic="a-fig-b" options="width=28.45274pt,height=28.45274pt" xml:id="p1.g1"/><graphics graphic="fig_trans" options="width=28.45274pt,height=28.45274pt" xml:id="p1.g2"/></para>"##;

/// graphics-tikz/gincltex_subfile_preamble_skipped: beside standalone, a sub-file gincltex boxes for
/// `\includegraphics{x.tex}` has its preamble gobbled as `\sa@documentclass` does (standalone.sty:602-646), so its
/// `\usepackage{cleveref}` and `\title` never run in the main document's body (2512.06733).
#[test]
fn gincltex_subfile_preamble_is_skipped() {
  let (log, xml) = latexml::util::test::convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/gincltex_subfile_preamble_skipped.tex"
    ),
    &[(
      "gincltex_sub.tex",
      include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/gincltex_sub.tex"),
    )],
    Some(AR5IV),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("SUBTITLE"), "{xml}");
  latexml::util::test::assert_element(&xml, "figure", &[r#"xml:id="S1.F1""#], FIGURE);
}

const FIGURE: &str = r##"<figure inlist="lof" xml:id="S1.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p xml:id="S1.F1.1"><text xml:id="S1.F1.1.1"><inline-block depth="0.0pt" height="7.9pt" width="69.0pt" xml:id="S1.F1.1.1.1" xscale="1.15265144128747" xtranslate="4.6pt" yscale="1.15265144128747" ytranslate="-0.5pt"><p xml:id="S1.F1.1.1.1.1"><text xml:id="S1.F1.1.1.1.1.1"><text width="0.0pt" xml:id="S1.F1.1.1.1.1.1.1"><inline-block vattach="bottom" xml:id="S1.F1.1.1.1.1.1.1.1"><p xml:id="S1.F1.1.1.1.1.1.1.1.1">SUBBODY</p></inline-block></text></text></p></inline-block></text></p><toccaption><tag close=" ">1</tag>Cap.</toccaption><caption><tag close=": ">Figure 1</tag>Cap.</caption></figure>"##;

/// alignment-bindings/backslashbox_reads_its_arguments_in_the_cell: diagbox.sty:216-221's `\backslashbox` begins with
/// a `\def`, so an alignment's cell-head peek (tex.web §791) stops there and `\backslashbox{Model} & x`'s second
/// argument is the cell's v-part, not the `&` (1106.0505).
#[test]
fn backslashbox_reads_its_arguments_in_the_cell() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/backslashbox_reads_its_arguments_in_the_cell.tex"
    ),
    AR5IV,
    (0, 0),
    &[("tabular", "p1.1", BACKSLASHBOX_TABULAR)],
  );
}

const BACKSLASHBOX_TABULAR: &str = r##"<tabular vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="center" class="ltx_nopad" xml:id="p1.1.1.1"><picture height="9.61" width="75.34" xml:id="p1.pic1"><line points="0,9.61 75.34,0" stroke="#000000" stroke-width="0.4"/><g class="ltx_svg_fog" innerheight="9.61" innerwidth="37.67" transform="translate(0,0)"><inline-block xml:id="p1.pic1.1"><inline-block align="left" xml:id="p1.pic1.1.1"><p xml:id="p1.pic1.1.1.1">Model</p></inline-block></inline-block></g><g class="ltx_svg_fog" innerheight="0" innerwidth="0" transform="translate(75.34,9.61)"><inline-block xml:id="p1.pic1.2"><inline-block align="right" xml:id="p1.pic1.2.1"><p xml:id="p1.pic1.2.1.1"/></inline-block></inline-block></g></picture></td><td align="center" xml:id="p1.1.1.2">x</td></tr><tr xml:id="p1.1.2"><td align="center" xml:id="p1.1.2.1">y</td><td align="center" xml:id="p1.1.2.2">z</td></tr></tbody></tabular>"##;
