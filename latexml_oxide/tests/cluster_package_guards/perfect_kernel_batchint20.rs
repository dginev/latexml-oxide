//! Red/green guards for the int20 fix round (the 70a + 70b bundle A/B's worse rows): AMS-TeX's `\Let@` sets the row
//! end to `\cr` (amstex.tex:486), graphicx reads a graphic's name at the document's catcodes, a sub-file gincltex boxes
//! has its preamble skipped beside standalone, and `\backslashbox` reads its arguments inside the cell.
//!
//! Every repro is checked against pdflatex/pdftex (its header holds the oracle).
use super::perfect_kernel_batch57::assert_elements;

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
