//! FIX991: an absent operand's placeholder is an Ord atom, so MathJax spaces the relation after it (issue 991).
use super::perfect_kernel_batch46::{convert_html, error_count, warning_count};

/// Issue 991: every `absent` operand (the LHS of an `align`/`aligned` continuation cell, a prefix relation, an empty
/// `\frac` slot, `\sqrt{}`) is `<mphantom><mi>U+2060</mi></mphantom>`. MathJax spaces an operator by the TeX class of
/// the atom before it; an EMPTY `mphantom` has none and passes the row's start through, so the `=` after it got no
/// space; the `mi` is an Ord atom (amsmath's `{}` before the cell). Red on 108cb6dc9dd: `<mphantom></mphantom>` in each.
#[test]
fn absent_operand_is_a_phantom_word_joiner() {
  let ph = "<mphantom><mi>\u{2060}</mi></mphantom>";
  let (stderr, html) = convert_html(include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/absent_operand_is_an_ord_atom.tex"
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
/// was an empty `<mrow>` (an empty `<mpadded>`): no TeX class in MathJax, so the `=` beside it got no space on that
/// side, and the spacing walk zeroed `−`'s and `+`'s operator space as if unary. In TeX the box is a zero-width Ord
/// atom: beside an operator it is the absent operand's placeholder; in a script or a fraction slot, or beside an
/// invisible operator (no space to lose), it stays the empty `<mrow>`. A box of sized empty boxes keeps each `mpadded` with
/// its attributes, the placeholder added beside them (b1fd8d00277 dropped them: the `=` lost 2cm of offset); the
/// `\,` after an empty box moves onto the placeholder, so the `=` gets TeX's thin + thick space (wrapped and lost before).
/// Red on 8851f75996e: `<mrow></mrow>` (`<mpadded …></mpadded>`) in the first nine; on b1fd8d00277 the last three.
#[test]
fn empty_box_beside_an_operator_is_a_phantom_word_joiner() {
  let ph = "<mphantom><mi>\u{2060}</mi></mphantom>";
  let strut = "\\hbox{\\vrule height=8.39996pt,depth=3.60004pt,width=0.0pt}";
  let cm = r#"<mpadded style="width:28.5pt;" width="28.5pt"></mpadded>"#;
  let (stderr, html) = convert_html(include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/empty_box_is_an_ord_atom.tex"
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

/// Issue 991, the slot's U+2060 in a string value: the XSLT takes a footnote's mark as the string value of its tag, here
/// a formula with an absent operand (2509.25166's mn2e marks `$*\dagger$`), which leaked the slot's U+2060 into the
/// `<sup>`; `f:plain-text` (LaTeXML-common.xsl) drops it. The page title and the ref tooltips are UnicodeMath, taken
/// from the XMath before the MathML exists: no U+2060 to leak. Red on 9d5f3e45fe3: U+2060 before each mark's `=1`.
#[test]
fn absent_operand_leaves_no_word_joiner_in_a_string_value() {
  let (stderr, html) = convert_html(include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/absent_operand_in_a_string_value.tex"
  ));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // Every U+2060 is inside a `<math>` element (the slot), none in text or an attribute outside one.
  let outside: usize = html
    .split("<math")
    .enumerate()
    .map(|(i, part)| {
      let rest = if i == 0 {
        part
      } else {
        part.split_once("</math>").map_or("", |(_, r)| r)
      };
      rest.matches('\u{2060}').count()
    })
    .sum();
  assert_eq!(outside, 0, "{html}");
  let marks: Vec<&str> = html
    .split("<sup class=\"ltx_note_mark\">")
    .skip(1)
    .map(|s| s.split_once("</sup>").map_or(s, |(mark, _)| mark))
    .collect();
  assert_eq!(
    marks.len(),
    2,
    "the footnote's mark and its copy in the note:\n{html}"
  );
  for mark in marks {
    assert!(mark.trim_end().ends_with("=1"), "{mark:?}");
  }
  assert!(html.contains("<title>On =𝑏</title>"), "{html}");
  assert!(html.contains("title=\"1 Title =𝑏 ‣ On =𝑏\""), "{html}");
}

/// Issue 991, the continuation sign: amsmath sets an empty Ord before every right-hand cell of `{align}`/`{alignat}`/
/// `{aligned}`/`{split}` and every `{multline}` line (`\displaystyle{{}##}`, amsmath.sty:2347-2354, :1483-1486,
/// :2424-2425, :2580), so TeX sets the sign opening such a cell binary. Under the user's ruling of 2026-11-23 it is read
/// so where the cell continues the one before it — an empty left cell and no relation of its own, or a left cell ending
/// in an operand; a multline line after the first —: `plus(absent, c)`, the placeholder before the `<mo>`, the leading
/// `\quad` the `−`'s lspace. A left cell ending in a relation (`B =& -C`), a cell with its own relation (`&-\Delta u =
/// f`), the first multline line, gather and eqnarray cells (no `{}`) stay unary; `&= b`, `&\times c` were already
/// binary. The lexer paths: an `{align}` cell and its row's MathFork formula, a lifted `\[\begin{aligned}…\]` cell, an
/// XMArray cell (`{split}`), a multline line; and the prefix retry of a sign taking no operand (`& + \ \ {\rm complex\
/// conj.}`, cond-mat/0505096, an extra unparsed Warning without it). Red on bc12e3e8991: every one of these signs a
/// prefix (`+ c`).
#[test]
fn continuation_sign_after_amsmath_s_empty_ord_is_binary() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/math-parse/continuation_sign_is_binary.tex");
  // the parser's reading of each cell and row formula
  let (log, xml) = super::perfect_kernel_batch46::convert(tex, true);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for (id, text) in [
    ("S0.E1.m2", "absent = b"),
    ("S0.E2.m1", "absent + c"),
    ("S0.E2.m2", "absent + c"),
    ("S0.E3.m1", "absent plus-or-minus c"),
    ("S0.E4.m1", "absent minus-or-plus c"),
    ("S0.E5.m1", "absent - d"),
    ("S0.E6.m1", "absent * c"),
    ("S0.E7.m1", "(absent + c) - d"),
    ("S0.Ex2.m1", "absent - c"),
    ("S0.E8.m1", "- a"),
    ("S0.E8.m3", "- a = b"),
    ("S0.E9.m2", "absent + c"),
    ("S0.E9.m3", "a = b + c"),
    ("S0.E10.m1", "B = absent"),
    ("S0.E10.m2", "- C"),
    ("S0.E11.m1", "- Delta * u = f"),
    ("S0.E11.m2", "- Delta * u = f"),
    ("S0.E13.m1", "+ c"),
    ("S0.E15.m1", "+ g"),
    ("S0.Ex3Xa.m2", "absent + c"),
    ("S0.E16.m1", "a = b - c"),
    ("S0.E17.m1", "(- a) + b + c"),
    ("S0.E18.m3", "- 1 &lt; x &lt; 0"),
    ("S0.E19.m1", "absent + c"),
    ("S0.E19.m2", "absent - d"),
    // a sign taking no operand, which no continuation reading parses, is the prefix sign it was (cond-mat/0505096)
    ("S0.E21.m1", "fragments@(+, complex * conj)"),
  ] {
    assert!(
      xml.contains(&format!("text=\"{text}\" xml:id=\"{id}\"")),
      "{id}: {text}\n{xml}"
    );
  }
  // the presentation of each cell
  let (stderr, html) = convert_html(tex);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let ph = "<mphantom><mi>\u{2060}</mi></mphantom>";
  let binary =
    |op: &str, operand: &str| format!("<mrow>{ph}<mo>{op}</mo><mi>{operand}</mi></mrow>");
  let unary = |op: &str, operand: &str| format!("<mrow><mo>{op}</mo><mi>{operand}</mi></mrow>");
  for (id, tex, body) in [
    ("S0.E1.m2", "\\displaystyle=b", binary("=", "b")),
    ("S0.E2.m1", "\\displaystyle+c", binary("+", "c")),
    ("S0.E3.m1", "\\displaystyle\\pm c", binary("±", "c")),
    ("S0.E4.m1", "\\displaystyle\\mp c", binary("∓", "c")),
    (
      "S0.E5.m1",
      "\\displaystyle\\quad-d",
      format!(r#"<mrow>{ph}<mo lspace="1.222em">−</mo><mi>d</mi></mrow>"#),
    ),
    (
      "S0.E6.m1",
      "\\displaystyle\\times c",
      format!(r#"<mrow>{ph}<mo lspace="0.222em" rspace="0.222em">×</mo><mi>c</mi></mrow>"#),
    ),
    (
      "S0.E7.m1",
      "\\displaystyle+c-d",
      format!("<mrow>{}<mo>−</mo><mi>d</mi></mrow>", binary("+", "c")),
    ),
    ("S0.Ex2.m1", "\\displaystyle-c", binary("−", "c")),
    ("S0.E8.m1", "\\displaystyle-a", unary("−", "a")),
    ("S0.E9.m2", "\\displaystyle+c", binary("+", "c")),
    ("S0.E10.m2", "\\displaystyle-C", unary("−", "C")),
    ("S0.E15.m1", "\\displaystyle+g", unary("+", "g")),
    ("S0.Ex3Xa.m2", "\\displaystyle+c", binary("+", "c")),
    ("S0.E19.m1", "\\displaystyle+c", binary("+", "c")),
    ("S0.E19.m2", "\\displaystyle-d", binary("−", "d")),
  ] {
    assert_math(&html, id, tex, &body);
  }
  // a gather line stays unary; a split's continuation cell and a multline's second line are binary, its first unary
  latexml::util::test::assert_element(
    &html,
    "math",
    &["id=\"S0.E13.m1\""],
    &format!(
      r#"<math id="S0.E13.m1" class="ltx_Math" alttext="\displaystyle+c" display="block">{}</math>"#,
      unary("+", "c")
    ),
  );
  assert!(
    html.contains(&format!(
      r#"<mtd></mtd><mtd class="ltx_align_left" columnalign="left">{}</mtd>"#,
      binary("−", "c")
    )),
    "{html}"
  );
  assert!(
    html.contains(&format!(
      r#"<mtd class="ltx_align_left" columnalign="left"><mrow>{}<mo>+</mo><mi>b</mi></mrow></mtd></mtr><mtr><mtd class="ltx_align_right" columnalign="right">{}</mtd>"#,
      unary("−", "a"),
      binary("+", "c")
    )),
    "{html}"
  );
}
