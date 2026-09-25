//! Batch 56ce: `\hyperlink{name}{text}` digests its text in its own group
//! (Perl hyperref.sty.ltxml:234 `bounded => 1`), so a font switch inside the
//! link ends with it — abntexto.tex:61 `\hyperlink{…}{\color{blue}\ttfamily
//! \tmp}` leaked `\ttfamily` into the rest of the document and its bibliography.
//! The whole paragraph is pinned: typewriter inside the link only.

#[test]
fn hyperlink_text_is_a_bounded_group() {
  let tex = "\\documentclass{article}\n\\usepackage{hyperref}\n\\begin{document}\n\\hyperlink{tgt}{\\ttfamily link} after\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    "<p><ref font=\"typewriter\" idref=\"tgt\">link</ref> after</p>",
  );
}
