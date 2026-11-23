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
    assert_math(&html, id, tex, &body);
  }
}

/// The whole inline `<math>` element of id `id`.
fn assert_math(html: &str, id: &str, tex: &str, body: &str) {
  latexml::util::test::assert_element(
    html,
    "math",
    &[&format!("id=\"{id}\"")],
    &format!(r#"<math id="{id}" class="ltx_Math" alttext="{tex}" display="inline">{body}</math>"#),
  );
}

/// Issue 991, the empty box: `\mbox{}`, `\hbox{}`, `\text{}`, `\strut`, `\makebox[0pt]{}` are an empty `XMText`, which
/// was an empty `<mrow>` (an empty `<mpadded>`): space-like, so the `=` beside it took the prefix (postfix) form, and
/// the spacing walk zeroed `−`'s and `+`'s operator space as if unary. In TeX the box is a zero-width Ord atom: beside
/// an operator it is the absent operand's placeholder; in a script or a fraction slot, or beside an invisible operator
/// (spaced alike in every form), it stays the empty `<mrow>`. A box of sized empty boxes keeps each `mpadded` with
/// its attributes, the placeholder added beside them (b1fd8d00277 dropped them: the `=` lost 2cm of offset); the
/// `\,` after an empty box moves onto the placeholder, so the `=` gets TeX's thin + thick space (wrapped and lost before).
/// Red on 8851f75996e: `<mrow></mrow>` (`<mpadded …></mpadded>`) in the first nine; on b1fd8d00277 the last three.
#[test]
fn empty_box_beside_an_operator_is_a_phantom_word_joiner() {
  let ph = "<mphantom><mi>\u{2060}</mi></mphantom>";
  let strut = "\\hbox{\\vrule height=8.39996pt,depth=3.60004pt,width=0.0pt}";
  let cm = r#"<mpadded style="width:28.5pt;" width="28.5pt"></mpadded>"#;
  let (stderr, html) = convert_html(include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/empty_box_is_not_space_like.tex"
  ));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let eq_b = format!("<mrow>{ph}<mo>=</mo><mi>b</mi></mrow>");
  for (id, tex, body) in [
    ("p1.m1", "\\mbox{}=b".to_string(), eq_b.clone()),
    ("p1.m2", "\\hbox{}=b".to_string(), eq_b.clone()),
    ("p1.m3", "\\text{}=b".to_string(), eq_b.clone()),
    ("p1.m4", format!("{strut}=b"), eq_b.clone()),
    (
      "p1.m5",
      format!("a={strut}"),
      format!("<mrow><mi>a</mi><mo>=</mo>{ph}</mrow>"),
    ),
    (
      "p1.m6",
      "\\makebox[0.0pt]{}=b".to_string(),
      format!(
        r#"<mrow><mpadded style="width:0.0pt;" width="0.0pt">{ph}</mpadded><mo>=</mo><mi>b</mi></mrow>"#
      ),
    ),
    (
      "p1.m7",
      "\\mbox{}-b".to_string(),
      format!("<mrow>{ph}<mo>−</mo><mi>b</mi></mrow>"),
    ),
    (
      "p1.m8",
      "a+\\mbox{}".to_string(),
      format!("<mrow><mi>a</mi><mo>+</mo>{ph}</mrow>"),
    ),
    ("p1.m9", "\\mbox{}\\mbox{}=b".to_string(), eq_b),
    (
      "p1.m10",
      "x^{\\mbox{}}=b".to_string(),
      "<mrow><msup><mi>x</mi><mrow></mrow></msup><mo>=</mo><mi>b</mi></mrow>".to_string(),
    ),
    (
      "p1.m11",
      "\\frac{\\mbox{}}{b}".to_string(),
      "<mfrac><mrow></mrow><mi>b</mi></mfrac>".to_string(),
    ),
    (
      "p1.m12",
      "\\mbox{}x".to_string(),
      "<mrow><mrow></mrow><mo>\u{2062}</mo><mi>x</mi></mrow>".to_string(),
    ),
    (
      "p1.m13",
      "\\mbox{\\makebox[28.45274pt]{}\\makebox[28.45274pt]{}}=b".to_string(),
      format!("<mrow><mrow>{cm}{cm}{ph}</mrow><mo>=</mo><mi>b</mi></mrow>"),
    ),
    (
      "p1.m14",
      "\\raisebox{4.30554pt}{\\makebox[28.45274pt]{}\\makebox[28.45274pt]{}}=b".to_string(),
      format!(
        r#"<mrow><mpadded voffset="4.3pt">{cm}{cm}{ph}</mpadded><mo>=</mo><mi>b</mi></mrow>"#
      ),
    ),
    (
      "p1.m15",
      "\\mbox{}\\,=b".to_string(),
      format!(r#"<mrow>{ph}<mo lspace="0.448em">=</mo><mi>b</mi></mrow>"#),
    ),
  ] {
    assert_math(&html, id, &tex, &body);
  }
}
