//! Red/green guards for batch NULLDELIM: TeX's null delimiter (`\left( a\right.`, `\left. a\right)`) prints nothing.
//! The parser's fence retry (`balance_null_delimiters`, OXIDIZED_DESIGN #67) re-supplies it as an empty fence token
//! so the formula parses; that token is the parser's scaffolding and leaves the tree once the parse has used it, as
//! Perl's XMath has no token for the `.` (TeX_Math.pool.ltxml `\lx@delimiterdot` is an XMHint). Kept, it was drawn
//! as `<mo>CLOSE</mo>` (`OPEN`) by the empty-token failsafe (Perl MathML.pm:708-714). Repro
//! `tools/perfect_kernel/repros/math-parse/null_delimiter_draws_nothing.tex`; witnesses 2606.13010, 1501.07594,
//! 2602.05799, 1007.4418.
use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

const REPRO: &str =
  include_str!("../../../tools/perfect_kernel/repros/math-parse/null_delimiter_draws_nothing.tex");

/// Every `<math …>…</math>` element of an HTML page, in document order.
fn math_elements(html: &str) -> Vec<&str> {
  html
    .match_indices("<math ")
    .map(|(at, _)| {
      let end = at + html[at..].find("</math>").expect("a closed <math>") + "</math>".len();
      &html[at..end]
    })
    .collect()
}

/// The core XML keeps no token for a null delimiter: the fence retry's own is gone, the written delimiter and the
/// parse (`delimited-(`) stay. One guard for the mechanism; the shapes share it.
#[test]
fn null_delimiter_leaves_no_token() {
  let (log, xml) = convert_with(REPRO, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  // The controls `\left[ x\right.` and `\left| x\right.` stay unparsed, as in Perl: one unparsed_math each.
  assert_eq!(warning_count(&log), 2, "{log}");
  assert_eq!(log.matches("Warning:unparsed_math:").count(), 2, "{log}");
  for empty in [r#"<XMTok role="CLOSE"/>"#, r#"<XMTok role="OPEN"/>"#] {
    assert!(!xml.contains(empty), "{empty}: {xml}");
  }
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="Sx1.p1.m1""#],
    concat!(
      r#"<Math mode="inline" tex="\left(a+b\right." text="delimited-(@(a + b)" xml:id="Sx1.p1.m1">"#,
      r#"<XMath xml:id="Sx1.p1.m1.2"><XMDual xml:id="Sx1.p1.m1.2.1"><XMApp xml:id="Sx1.p1.m1.2.1.1">"#,
      r#"<XMTok meaning="delimited-("/><XMRef idref="Sx1.p1.m1.1" xml:id="Sx1.p1.m1.2.1.1.2"/></XMApp>"#,
      r#"<XMWrap xml:id="Sx1.p1.m1.2.1.2"><XMTok role="OPEN" stretchy="true">(</XMTok>"#,
      r#"<XMApp xml:id="Sx1.p1.m1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok>"#,
      r#"<XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp>"#,
      r#"</XMWrap></XMDual></XMath></Math>"#,
    ),
  );
}

/// The MathML draws the written delimiter and nothing for the null one (Perl: `<mrow><mo>(</mo><mi>a</mi></mrow>`),
/// on the right (`\left( a\right.`), on the left (`\left. a\right)`), beside `\middle`, and in both cells of a fence
/// split across an align break. Base RC3A: `<mo fence="true" lspace="0em">CLOSE</mo>` / `…rspace="0em">OPEN</mo>`.
#[test]
fn null_delimiter_draws_nothing() {
  let html = latexml::util::test::convert_to_html_page(REPRO).expect("HTML");
  let maths = math_elements(&html);
  for math in &maths {
    let text: String = math
      .split('<')
      .filter_map(|s| s.split_once('>'))
      .map(|(_, t)| t)
      .collect();
    assert!(!text.contains("OPEN") && !text.contains("CLOSE"), "{math}");
  }
  let inline = |id: &str, tex: &str, body: &str| {
    format!(r#"<math id="{id}" class="ltx_Math" alttext="{tex}" display="inline">{body}</math>"#)
  };
  let a_plus_b = "<mrow><mi>a</mi><mo>+</mo><mi>b</mi></mrow>";
  let f_of_x = concat!(
    "<mrow><mi>f</mi><mo>\u{2061}</mo>",
    r#"<mrow><mo stretchy="false">(</mo><mi>x</mi><mo stretchy="false">)</mo></mrow></mrow>"#
  );
  let expected = [
    inline(
      "Sx1.p1.m1",
      r"\left(a+b\right.",
      &format!("<mrow><mo>(</mo>{a_plus_b}</mrow>"),
    ),
    inline(
      "Sx1.p1.m2",
      r"\left(a\right.",
      "<mrow><mo>(</mo><mi>a</mi></mrow>",
    ),
    inline(
      "Sx1.p1.m3",
      r"\left(f(x)\right.",
      &format!("<mrow><mo>(</mo>{f_of_x}</mrow>"),
    ),
    inline(
      "Sx1.p1.m4",
      r"\left(\frac{a}{b}\right.",
      "<mrow><mo>(</mo><mfrac><mi>a</mi><mi>b</mi></mfrac></mrow>",
    ),
    inline(
      "Sx1.p1.m5",
      r"y=\left(a\right.",
      "<mrow><mi>y</mi><mo>=</mo><mrow><mo>(</mo><mi>a</mi></mrow></mrow>",
    ),
    inline(
      "Sx1.p1.m6",
      r"\left.a\right)",
      "<mrow><mi>a</mi><mo>)</mo></mrow>",
    ),
    inline(
      "Sx1.p1.m7",
      r"\left[a,b\right.",
      "<mrow><mo>[</mo><mi>a</mi><mo>,</mo><mi>b</mi></mrow>",
    ),
    inline(
      "Sx1.p1.m8",
      r"\left.a,b\right]",
      "<mrow><mi>a</mi><mo>,</mo><mi>b</mi><mo>]</mo></mrow>",
    ),
    inline(
      "Sx1.p1.m9",
      r"\left(a\middle|b\right.",
      r#"<mrow><mo>(</mo><mi>a</mi><mo lspace="0em" rspace="0em" stretchy="true">|</mo><mi>b</mi></mrow>"#,
    ),
    inline("Sx1.Ex1.m1", r"\displaystyle H", "<mi>H</mi>"),
    inline(
      "Sx1.Ex1.m2",
      r"\displaystyle=\frac{\hbar}{2}\left(a+b\right.",
      &format!(
        concat!(
          r#"<mrow><mphantom></mphantom><mo>=</mo><mrow><mstyle displaystyle="true"><mfrac>"#,
          r#"<mi mathvariant="normal">ℏ</mi><mn>2</mn></mfrac></mstyle><mo>{}</mo><mrow><mo>(</mo>{}</mrow></mrow></mrow>"#
        ),
        '\u{2062}', a_plus_b
      ),
    ),
    inline(
      "Sx1.E1.m1",
      r"\displaystyle\quad\left.+c+d\right).",
      concat!(
        "<mrow><mrow><mrow><mrow><mo>+</mo><mi>c</mi></mrow><mo>+</mo><mi>d</mi></mrow><mo>)</mo></mrow>",
        r#"<mo lspace="0em">.</mo></mrow>"#
      ),
    ),
  ];
  assert_eq!(
    maths[..expected.len()],
    expected.iter().map(String::as_str).collect::<Vec<_>>(),
    "{html}"
  );
  // Controls, never on the retry's path: an unparsed `\left[ x\right.` (as Perl) and a balanced fence.
  assert_eq!(
    maths[expected.len()],
    r#"<math id="Sx2.p1.m1" class="ltx_math_unparsed" alttext="\left[x\right." display="inline"><mrow><mo>[</mo><mi>x</mi></mrow></math>"#
  );
  assert_eq!(
    maths.last().copied(),
    Some(
      inline(
        "Sx2.p1.m4",
        r"\left(a+b\right)",
        &format!("<mrow><mo>(</mo>{a_plus_b}<mo>)</mo></mrow>")
      )
      .as_str()
    )
  );
}
