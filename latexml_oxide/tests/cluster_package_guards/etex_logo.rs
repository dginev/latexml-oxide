//! Batch 56cq: etex.sty:172 defines the `\eTeX` logo beside its register
//! allocation; the binding replacing it lacked the macro (Perl's
//! etex.sty.ltxml lacks it too). Only a document that loads etex reaches
//! it — bibleref-parse's manual uses `\eTeX` without the package and is
//! undefined under pdflatex as well.
#[test]
fn etex_logo_is_defined() {
  let tex = "\\documentclass{article}\n\\usepackage{etex}\n\\begin{document}\nNeeds an \\eTeX{} engine.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r##"<para xml:id="p1"><p>Needs an <Math mode="inline" tex="\varepsilon" text="varepsilon" xml:id="p1.m1"><XMath><XMTok font="italic" name="varepsilon" role="UNKNOWN">ε</XMTok></XMath></Math>-<text class="ltx_TeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.2em">T<text cssstyle="font-variant:small-caps;font-size:120%;" yoffset="-0.2ex">e</text>X</text> engine.</p></para>"##,
  );
}
