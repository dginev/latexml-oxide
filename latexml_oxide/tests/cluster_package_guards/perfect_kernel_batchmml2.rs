//! MML2: MathML Core spacing of the category-K operators (issue 789); MHCHEM4: a Bin TeX makes an Ord (issue 824).
use super::perfect_kernel_batch46::{convert_html, error_count, warning_count};

/// Issue 789: TeX's `/` and `\backslash` are Ords (no glue in `\showbox`); MathML Core puts infix `/`, `\` in
/// category K (zero space), but Chrome spaces `/` by an older dictionary, so the zero is explicit. Red on
/// fc5211bb491: `<mo>/</mo>` and `<mo lspace="0.222em" rspace="0.222em">\</mo>`. A kept Rel/Punct class keeps its
/// TeX space (`\showbox`: `\thickmuskip` each side, `\thinmuskip` after), as on fc5211bb491; red on 00361d99397
/// (revMML2): 0em / 0em.
#[test]
fn core_k_operators_have_no_operator_space() {
  let k = r#"<mo lspace="0em" rspace="0em">"#;
  let rel = r#"<mo lspace="0.278em" rspace="0.278em">"#;
  let punct = r#"<mo lspace="0em" rspace="0.167em" separator="true">"#;
  let slash = [
    (
      "p1.m1",
      "a/b",
      format!("<mrow><mi>a</mi>{k}/</mo><mi>b</mi></mrow>"),
    ),
    (
      "p1.m2",
      "G/H",
      format!("<mrow><mi>G</mi>{k}/</mo><mi>H</mi></mrow>"),
    ),
    (
      "p1.m3",
      "x^{1/2}",
      format!("<msup><mi>x</mi><mrow><mn>1</mn>{k}/</mo><mn>2</mn></mrow></msup>"),
    ),
    (
      "p1.m4",
      "a\\backslash b",
      format!("<mrow><mi>a</mi>{k}\\</mo><mi>b</mi></mrow>"),
    ),
    (
      "p1.m5",
      "a\\mathrel{/}b",
      format!("<mrow><mi>a</mi>{rel}/</mo><mi>b</mi></mrow>"),
    ),
    (
      "p1.m6",
      "a\\mathpunct{/}b",
      format!("<mrow><mi>a</mi>{punct}/</mo><mi>b</mi></mrow>"),
    ),
    (
      "p1.m7",
      "a\\mathrel{\\backslash}b",
      format!("<mrow><mi>a</mi>{rel}\\</mo><mi>b</mi></mrow>"),
    ),
    (
      "p1.m8",
      "a\\mathpunct{\\backslash}b",
      format!("<mrow><mi>a</mi>{punct}\\</mo><mi>b</mi></mrow>"),
    ),
  ];
  // MHCHEM4: tex.web §728-729 makes a Bin first, after Bin/Op/Rel/Open/Punct, before Rel/Close/Punct or last an Ord;
  // `\showbox` (TL 2025) puts \medmuskip both sides of the `+` of `a+-b`, none around its `−`. Red on 67f4e2564d3
  // (MHCHEM2): `<mo lspace="0em" rspace="0em">+</mo>` and a bare `<mo>−</mo>` in `a+-b` and `x\pm-1` (a limit-from rule).
  let bin = [
    (
      "p1.m1",
      "a+-b",
      format!("<mrow><mrow><mi>a</mi><mo>+</mo></mrow>{k}−</mo><mi>b</mi></mrow>"),
    ),
    (
      "p1.m2",
      "x\\pm-1",
      format!("<mrow><mrow><mi>x</mi><mo>±</mo></mrow>{k}−</mo><mn>1</mn></mrow>"),
    ),
    (
      "p1.m3",
      "x^{2+}",
      format!("<msup><mi>x</mi><mrow><mn>2</mn>{k}+</mo></mrow></msup>"),
    ),
    (
      "p1.m4",
      "f(0+)",
      format!(
        "<mrow><mi>f</mi><mo>\u{2061}</mo><mrow><mo stretchy=\"false\">(</mo><mrow><mn>0</mn>{k}+</mo></mrow><mo \
         stretchy=\"false\">)</mo></mrow></mrow>"
      ),
    ),
    (
      "p1.m5",
      "a+=b",
      format!("<mrow><mrow><mi>a</mi>{k}+</mo></mrow><mo>=</mo><mi>b</mi></mrow>"),
    ),
    (
      "p1.m6",
      "a+b+",
      format!("<mrow><mi>a</mi><mo>+</mo><mrow><mi>b</mi>{k}+</mo></mrow></mrow>"),
    ),
    (
      "p1.m7",
      "x+,y",
      format!("<mrow><mrow><mi>x</mi>{k}+</mo></mrow><mo>,</mo><mi>y</mi></mrow>"),
    ),
    (
      "p1.m8",
      "\\lim_{x\\to 0+}f",
      format!(
        "<mrow><msub><mo>lim</mo><mrow><mi>x</mi><mo stretchy=\"false\">→</mo><mrow><mn>0</mn>{k}+</mo></mrow></mrow></\
         msub><mi>f</mi></mrow>"
      ),
    ),
    ("p1.m9", "-x", "<mrow><mo>−</mo><mi>x</mi></mrow>".to_string()),
    (
      "p1.m10",
      "(-x)",
      "<mrow><mo stretchy=\"false\">(</mo><mrow><mo>−</mo><mi>x</mi></mrow><mo stretchy=\"false\">)</mo></mrow>"
        .to_string(),
    ),
    // `∇` is TeX's Ord: the `⋅` stays a Bin (its 0em left is the role table's Op-Bin, Perl's residual).
    (
      "p1.m11",
      "\\nabla\\cdot B",
      "<mrow><mo>∇</mo><mo lspace=\"0em\" rspace=\"0.222em\">⋅</mo><mi>B</mi></mrow>".to_string(),
    ),
    (
      "S0.E1.m2",
      "\\displaystyle=b+",
      format!("<mrow><mphantom></mphantom><mo>=</mo><mrow><mi>b</mi>{k}+</mo></mrow></mrow>"),
    ),
  ];
  for (repro, rows) in [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/math-parse/slash_is_an_ord_without_operator_space.tex"
      ),
      &slash[..],
    ),
    (
      include_str!("../../../tools/perfect_kernel/repros/math-parse/bin_atom_becomes_ord.tex"),
      &bin[..],
    ),
  ] {
    let (stderr, html) = convert_html(repro);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(warning_count(&stderr), 0, "{stderr}");
    for (id, tex, body) in rows {
      latexml::util::test::assert_element(
        &html,
        "math",
        &[&format!("id=\"{id}\"")],
        &format!(
          r#"<math id="{id}" class="ltx_Math" alttext="{tex}" display="inline">{body}</math>"#
        ),
      );
    }
  }
}
