//! FIX991: an absent operand's placeholder is not space-like, so the relation after it stays infix (issue 991).
use super::perfect_kernel_batch46::{convert_html, error_count, warning_count};

/// Issue 991: every `absent` operand (the LHS of an `align`/`aligned` continuation cell, a prefix relation, an empty
/// `\frac` slot, `\sqrt{}`) is `<mphantom><mi>U+2060</mi></mphantom>`. An EMPTY `mphantom` is space-like (MathML Core
/// §3.2.5.1), so the `=` after it took the prefix form in MathJax; an `mi` never is. Red on 108cb6dc9dd:
/// `<mphantom></mphantom>` in each.
#[test]
fn absent_operand_is_a_phantom_word_joiner() {
  let ph = "<mphantom><mi>\u{2060}</mi></mphantom>";
  let (stderr, html) = convert_html(include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/absent_operand_is_not_space_like.tex"
  ));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, tex, body) in [
    (
      "S0.E1.m2",
      "\\displaystyle=b",
      format!("<mrow>{ph}<mo>=</mo><mi>b</mi></mrow>"),
    ),
    (
      "S0.E2.m2",
      "\\displaystyle=d",
      format!("<mrow>{ph}<mo>=</mo><mi>d</mi></mrow>"),
    ),
    (
      "S0.Ex1X.m3",
      "\\displaystyle\\leq y",
      format!("<mrow>{ph}<mo>≤</mo><mi>y</mi></mrow>"),
    ),
    (
      "S0.Ex1Xa.m2",
      "\\displaystyle=z",
      format!("<mrow>{ph}<mo>=</mo><mi>z</mi></mrow>"),
    ),
    (
      "p1.m1",
      "=b",
      format!("<mrow>{ph}<mo>=</mo><mi>b</mi></mrow>"),
    ),
    (
      "p1.m2",
      "\\frac{a}{}",
      format!("<mfrac><mi>a</mi>{ph}</mfrac>"),
    ),
    (
      "p1.m3",
      "\\frac{}{b}",
      format!("<mfrac>{ph}<mi>b</mi></mfrac>"),
    ),
    ("p1.m4", "\\sqrt{}", format!("<msqrt>{ph}</msqrt>")),
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
