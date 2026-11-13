//! Batch 56cr: wrapstuff's box is placed from the LaTeX2e paragraph hooks
//! (`para/begin`/`para/end`, wrapstuff.sty:334/:539-552/:1905-1939) that
//! neither engine models, so the raw package built the box and never
//! emitted it — caption and content lost at 0 errors. The binding models
//! the environment as wrapfig's inline float (`wrapstuff_sty.rs`).

/// `type=table` → a captioned `<table>` floated right (the default side)
/// with the wrap width; an untyped box → a plain `<float>`; the
/// surrounding text keeps its paragraphs; the package's `\wrapstuffset`
/// and `\wrapstuffclear` (wrapstuff.sty:2516-2521) are defined.
#[test]
fn wrapped_box_is_an_inline_float() {
  if !latexml::util::test::kpse_has("wrapstuff.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{caption}\n\\usepackage{wrapstuff}\n\\wrapstuffset{ratio=0.6,leftsep=2em}\n\\begin{document}\n\\begin{wrapstuff}[type=table,width=3cm]\n\\caption{X}\n\\begin{tabular}{lr}a & 1\\end{tabular}\n\\end{wrapstuff}\nSome text here.\\wrapstuffclear\n\\begin{wrapstuff}[l]\\fbox{plain}\\end{wrapstuff}\nLast text.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "document",
    &[],
    r##"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><table float="right" width="24%" xml:id="tab1"><toccaption><tag close=" ">1</tag>X</toccaption><caption><tag close=": ">Table 1</tag>X</caption><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="right">1</td></tr></tbody></tabular></table><para xml:id="p1"><p>Some text here.</p></para><float float="left" xml:id="tab2"><p><text cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle">plain</text></p></float><para xml:id="p2"><p>Last text.</p></para></document>"##,
  );
}
