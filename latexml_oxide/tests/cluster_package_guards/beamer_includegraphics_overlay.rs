//! Batch 56cu: beamerbasemisc.sty:218-238 wraps `\includegraphics` so a
//! leading `<overlay>` is consumed before graphicx's original reads its
//! arguments; without the wrapper (Perl's beamer.cls.ltxml lacks it too)
//! `<` became the graphic and `1>file` leaked as text (babybeamer10 and
//! nine more beamer manuals).

/// `\includegraphics<1>{none}` is graphicx's `\includegraphics{none}`
/// (the overlay discarded: one pass, D14); the whole `<graphics>` element.
#[test]
fn includegraphics_overlay_specification_is_consumed() {
  let tex = "\\documentclass{beamer}\n\\begin{document}\n\\begin{frame}\n\\includegraphics<1>{none} and \\includegraphics<2->[width=2cm]{none}\n\\end{frame}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    !xml.contains(r#"graphic="&lt;""#),
    "the overlay's `<` became the graphic:\n{xml}"
  );
  latexml::util::test::assert_element(
    &xml,
    "document",
    &[],
    r##"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><subsection><para xml:id="p1"><graphics graphic="none" xml:id="p1.g1"/><p>and <graphics graphic="none" options="width=56.9055pt,keepaspectratio=true" xml:id="p1.g2"/></p></para></subsection></document>"##,
  );
}
