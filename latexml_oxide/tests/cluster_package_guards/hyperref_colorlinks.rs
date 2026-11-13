//! A bare `colorlinks` in `\hypersetup` or the package options is
//! `colorlinks=true` (hyperref.sty:3204 `\define@key{Hyp}{colorlinks}[true]`),
//! and hyperref then loads color (:4532-4536): ltnews's driver does
//! `\hypersetup{colorlinks}` and its issues use `\color`/`\textcolor`
//! without loading color themselves. Perl shares the `eq 'true'` guard
//! (hyperref.sty.ltxml:114); the whole paragraph is pinned.

#[test]
fn bare_colorlinks_loads_color() {
  let tex = "\\documentclass{article}\n\\usepackage{hyperref}\n\\hypersetup{colorlinks}\n\\begin{document}\nTest \\textcolor{red}{red} end.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>Test <text color="#FF0000">red</text> end.</p>"##,
  );
}
