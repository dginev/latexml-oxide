//! Red/green guards for batch 68z: alignment and kernel shapes from the rc_struct2 study (tex.web part 37 and the
//! real macros of each context). Item 1 — amsmath's display environments strip one enclosing group of their body, as
//! `\collect@@body#1\end#2` does (amsmath.sty:1424, tex.web §400; 2011.06919 2211.06952 1909.04856 2101.06401).
//! Item 2 — a column end goes back after the v-part as the token read, not its canonical `\cr` (Perl Gullet.pm:310;
//! hep-ph9612244).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

/// `xml` without its `tex="…"` attributes (the source the braces were part of).
fn without_tex(xml: &str) -> String {
  let mut out = String::with_capacity(xml.len());
  let mut rest = xml;
  while let Some(i) = rest.find(" tex=\"") {
    out.push_str(&rest[..i]);
    let after = &rest[i + 6..];
    rest = &after[after.find('"').map_or(after.len(), |j| j + 1)..];
  }
  out.push_str(rest);
  out
}

/// kernel-alignment/braced_amsmath_body: `\begin{align}{a&=b\\&=c}\end{align}` and its gather*, alignat, multline,
/// multline*, split and flalign* siblings typeset as without the braces (pdflatex 0 errors); a group that is not the
/// whole body (`{a}+b &= c`) keeps its braces. Before 68z: 3 Errors "Stray alignment".
#[test]
fn braced_amsmath_body_loses_its_group() {
  let braced =
    include_str!("../../../tools/perfect_kernel/repros/kernel-alignment/braced_amsmath_body.tex");
  let plain = r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\begin{align}a &= b \nonumber\\ &= c\end{align}
\begin{gather*}x \\ y\end{gather*}
\begin{alignat}{2}a &= b & c &= d\end{alignat}
\begin{multline*}a+b\\ +c\end{multline*}
\begin{multline}a+b\\ +c\end{multline}
\begin{equation}\begin{split}a &= b\\ &= c\end{split}\end{equation}
\begin{flalign*}a &= b\end{flalign*}
\begin{align*}{a}+b &= c\end{align*}
\end{document}
";
  let (stderr, xml) = convert_with(braced, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let (plain_stderr, plain_xml) = convert_with(plain, None);
  assert_eq!(error_count(&plain_stderr), 0, "{plain_stderr}");
  assert_eq!(without_tex(&xml), without_tex(&plain_xml));
  assert_element(
    &xml,
    "equationgroup",
    &["xml:id=\"S0.EGx2\""],
    r#"<equationgroup class="ltx_eqn_gather" xml:id="S0.EGx2"><equation xml:id="S0.Ex2"><Math mode="display" tex="\displaystyle x" text="x" xml:id="S0.Ex2.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></equation><equation xml:id="S0.Ex3"><Math mode="display" tex="\displaystyle y" text="y" xml:id="S0.Ex3.m1"><XMath><XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math></equation></equationgroup>"#,
  );
  assert_eq!(xml.matches("<td").count(), 12, "{xml}");
}

/// kernel-alignment/let_cr_ends_the_row_as_itself: `\let\crnorm=\cr \def\cr{&\crnorm}` and a template
/// `#&\vrule #\crnorm`: the `\vrule`'s keyword scan meets `\crnorm`, which ends the row (pdflatex 0 errors). Before 68z
/// the canonical `\cr` went back after the v-part, the scan expanded the macro and its `&` was 1 Error "Extra alignment
/// tab".
#[test]
fn a_let_cr_ends_the_row_as_itself() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/let_cr_ends_the_row_as_itself.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular><tr><td align="left" border="r" class="ltx_nopad_l ltx_nopad_r">A</td></tr></tabular>"#,
  );
}
