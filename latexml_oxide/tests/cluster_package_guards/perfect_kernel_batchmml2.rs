//! MML2: MathML Core spacing of the category-K operators (issue 789).
use super::perfect_kernel_batch46::{convert_html, error_count, warning_count};

/// Issue 789: TeX's `/` and `\backslash` are Ords (no glue in `\showbox`); MathML Core puts infix `/`, `\` in
/// category K (zero space), but Chrome spaces `/` by an older dictionary, so the zero is explicit. Red on
/// fc5211bb491: `<mo>/</mo>` and `<mo lspace="0.222em" rspace="0.222em">\</mo>`.
#[test]
fn core_k_operators_have_no_operator_space() {
  let (stderr, html) = convert_html(include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/slash_is_an_ord_without_operator_space.tex"
  ));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let k = r#"<mo lspace="0em" rspace="0em">"#;
  for (id, tex, body) in [
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
  ] {
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
