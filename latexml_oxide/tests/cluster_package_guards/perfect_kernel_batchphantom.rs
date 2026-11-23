//! Red/green guards for batch PHANTOM: a math phantom is a box of its content (DG ruling 2026-11-23), `<mphantom>` where
//! it stands, `\hphantom` without height and depth and `\vphantom` without width (`<mpadded>`); the parser still reads
//! it as space (OXIDIZED_DESIGN_DIVERGENCES #1141, KNOWN_PERL_ERRORS #1316). Witness 2603.21852 (`\sqrt{\phantom{x}}`).
//! Each guard converts the repro math-phantom/phantom_is_a_box_of_its_content (pdflatex: 0 errors) and asserts whole
//! `<math>` elements, the conversion free of errors and warnings.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const REPRO: &str = include_str!(
  "../../../tools/perfect_kernel/repros/math-phantom/phantom_is_a_box_of_its_content.tex"
);

/// The repro converted to HTML by the binary in a directory of its own, free of errors and warnings.
fn html() -> String {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), REPRO).expect("write t.tex");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  std::fs::read_to_string(workdir.path().join("t.html")).expect("read t.html")
}

fn assert_math(html: &str, id: &str, alttext: &str, body: &str) {
  assert_element(
    html,
    "math",
    &[&format!("id=\"{id}\"")],
    &format!(
      "<math id=\"{id}\" class=\"ltx_Math\" alttext=\"{alttext}\" display=\"inline\" intent=\":literal\">{body}</math>"
    ),
  );
}

/// A slot holding only a phantom (the parser's `absent` content, `dual_of_absent`): the radical, a script, a
/// numerator, a pre-script and `\hphantom` (zero height and depth) keep the box. Red (INT081C): an empty
/// `<mphantom/>` each, Perl `<mi></mi>`.
#[test]
fn phantom_alone_in_a_slot_is_its_presentation() {
  let html = html();
  assert_math(
    &html,
    "p1.m1",
    r"\sqrt{\phantom{x}}",
    "<msqrt><mphantom><mi>x</mi></mphantom></msqrt>",
  );
  assert_math(
    &html,
    "p3.m1",
    r"\sqrt{\hphantom{x}}",
    "<msqrt><mpadded style=\"height:0pt;vertical-align:-0pt;\" depth=\"0pt\" height=\"0pt\"><mphantom><mi>x</mi></mphantom>\
     </mpadded></msqrt>",
  );
  assert_math(
    &html,
    "p6.m1",
    r"x^{\phantom{2}}y",
    "<mrow><msup><mi>x</mi><mphantom><mn>2</mn></mphantom></msup><mo>\u{2062}</mo><mi>y</mi></mrow>",
  );
  assert_math(
    &html,
    "p7.m1",
    r"\frac{\phantom{x}}{y}",
    "<mfrac><mphantom><mi>x</mi></mphantom><mi>y</mi></mfrac>",
  );
  assert_math(
    &html,
    "p8.m1",
    r"{}^{\phantom{a}}b",
    "<mmultiscripts><mi>b</mi><mprescripts></mprescripts><mrow></mrow><mphantom><mi>a</mi></mphantom></mmultiscripts>",
  );
}

/// A phantom at a list's edge (`edge_phantoms`, `wrap_presentation`): a matrix cell's `\phantom{-}`, a
/// radicand's trailing `\phantom{x}`, a leading `\vphantom{\int}` strut (zero width). Between two items a phantom stays
/// their spacing (the `lspace` of `a\phantom{x}b`'s invisible times), and an empty `\phantom{}` is nothing, as before.
/// Red (INT081C): the edge phantoms lost.
#[test]
fn phantom_at_a_list_edge_keeps_its_place() {
  let html = html();
  let cell = "<mtd><mrow><mphantom><mo>\u{2212}</mo></mphantom><mn>1</mn></mrow></mtd>";
  let minus_one = "<mtd><mrow><mo>\u{2212}</mo><mn>1</mn></mrow></mtd>";
  assert_math(
    &html,
    "p5.m1",
    r"\begin{pmatrix}\phantom{-}1&amp;-1\\ -1&amp;\phantom{-}1\end{pmatrix}",
    &format!(
      "<mrow><mo>(</mo><mtable columnspacing=\"5pt\"><mtr>{cell}{minus_one}</mtr><mtr>{minus_one}{cell}</mtr></mtable>\
       <mo>)</mo></mrow>"
    ),
  );
  assert_math(
    &html,
    "p4.m1",
    r"\sqrt{a\phantom{x}}",
    "<msqrt><mrow><mi>a</mi><mphantom><mi>x</mi></mphantom></mrow></msqrt>",
  );
  assert_math(
    &html,
    "p9.m1",
    r"\vphantom{\int}a",
    "<mrow><mpadded style=\"width:0pt;\" width=\"0pt\"><mphantom><mo>\u{222b}</mo></mphantom></mpadded><mi>a</mi></mrow>",
  );
  assert_math(
    &html,
    "p10.m1",
    r"a\phantom{x}b",
    "<mrow><mi>a</mi><mo lspace=\"0.570em\">\u{2062}</mo><mi>b</mi></mrow>",
  );
  assert_math(&html, "p11.m1", r"\phantom{}x", "<mi>x</mi>");
}

/// A formula of a phantom alone is math (`has_real_math`), no longer unwrapped to spaces; text mode's phantom is
/// unchanged. Red (INT081C): `B ` with no `<math>`.
#[test]
fn phantom_alone_is_a_formula() {
  let html = html();
  assert_math(
    &html,
    "p2.m1",
    r"\phantom{x}",
    "<mphantom><mi>x</mi></mphantom>",
  );
  assert_element(
    &html,
    "span",
    &["id=\"p11.1.1\""],
    "<span id=\"p11.1.1\" class=\"ltx_text ltx_phantom\"><span style=\"visibility:hidden\">text</span></span>",
  );
}

/// The core: content keeps ignoring the phantom. A slot of a phantom alone is an XMDual whose content branch is
/// `absent`; a slot with an item keeps its `XMArg`, the hint (with its parsed argument) beside the item, no XMDual,
/// XMRef or shifted id (PHANTOM round 4, `item_beside_phantoms`; round 3 wrapped it in an XMDual).
#[test]
fn phantom_is_presentation_only_in_the_core() {
  let (log, xml) = convert_with(REPRO, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m1\""],
    r#"<Math mode="inline" tex="\sqrt{\phantom{x}}" text="square-root@(absent)" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="square-root"/><XMDual xml:id="p1.m1.1.1.2"><XMTok meaning="absent"/><XMHint depth="0.0pt" height="4.3pt" name="phantom" width="5.7pt" xml:id="p1.m1.1.1.2.2"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMHint></XMDual></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p4.m1\""],
    r#"<Math mode="inline" tex="\sqrt{a\phantom{x}}" text="square-root@(a)" xml:id="p4.m1"><XMath xml:id="p4.m1.1"><XMApp xml:id="p4.m1.1.1"><XMTok meaning="square-root"/><XMArg xml:id="p4.m1.1.1.2"><XMTok font="italic" role="UNKNOWN">a</XMTok><XMHint depth="0.0pt" height="4.3pt" name="phantom" width="5.7pt" xml:id="p4.m1.1.1.2.2"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMHint></XMArg></XMApp></XMath></Math>"#,
  );
}

const BESIDE_A_QUAD: &str = include_str!(
  "../../../tools/perfect_kernel/repros/math-phantom/phantom_beside_a_quad_is_padding.tex"
);

/// A phantom beside a `\quad` (`filter_hints_beside`), witness astro-ph0111392: a failed parse keeps the phantoms as
/// padding, the quad's width with theirs (A lpadding 17.8pt, B rpadding 15.7pt, as INT081C); a parsed one keeps the
/// quad its neighbour's padding, never a PUNCT (C). Red (PHANTOM round 2): A 7.8pt, B and C a PUNCT quad. The two
/// warnings are A's and B's unparsed math (the lone `\biggr]`).
#[test]
fn phantom_beside_a_quad_is_padding() {
  let (log, xml) = convert_with(BESIDE_A_QUAD, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m1\""],
    r#"<Math class="ltx_math_unparsed" mode="inline" tex="\phantom{{}={}}\quad{}+c\biggr]" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok lpadding="17.8pt" meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok fontsize="210%" role="CLOSE" stretchy="false">]</XMTok></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p2.m1\""],
    r#"<Math class="ltx_math_unparsed" mode="inline" tex="c\biggr]\quad\phantom{x}" xml:id="p2.m1"><XMath xml:id="p2.m1.1"><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok fontsize="210%" role="CLOSE" rpadding="15.7pt" stretchy="false">]</XMTok></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p3.m1\""],
    r#"<Math mode="inline" tex="a\quad\phantom{x}" text="a" xml:id="p3.m1"><XMath xml:id="p3.m1.1"><XMTok font="italic" role="UNKNOWN" rpadding="10.0pt">a</XMTok><XMHint depth="0.0pt" height="4.3pt" name="phantom" width="5.7pt" xml:id="p3.m1.1.2"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMHint></XMath></Math>"#,
  );
}

/// A phantom of several items takes the kludge's structure (`in_phantom`: its fence wrapped, its script attached), not
/// the grammar's, and the formula stays parsed. Distinct from the guards above: their phantoms hold one item.
#[test]
fn phantom_of_several_items_is_kludged() {
  let (log, xml) = convert_with(BESIDE_A_QUAD, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p4.m1\""],
    r#"<Math mode="inline" tex="\sqrt{a\phantom{x+(y)^{2}}}" text="square-root@(a)" xml:id="p4.m1"><XMath xml:id="p4.m1.1"><XMApp xml:id="p4.m1.1.1"><XMTok meaning="square-root"/><XMArg xml:id="p4.m1.1.1.2"><XMTok font="italic" role="UNKNOWN">a</XMTok><XMHint depth="2.5pt" height="8.1pt" name="phantom" width="31.0pt" xml:id="p4.m1.1.1.2.2"><XMArg xml:id="p4.m1.1.1.2.2.1"><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp xml:id="p4.m1.1.1.2.2.1.3"><XMTok role="SUPERSCRIPTOP" scriptpos="post2"/><XMWrap xml:id="p4.m1.1.1.2.2.1.3.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMArg></XMHint></XMArg></XMApp></XMath></Math>"#,
  );
}

/// `\mathstrut` is TeX's `\vphantom(` (latex.ltx:15602, plain.tex:1034; DG ruling 2026-11-23): a zero-width phantom of
/// a parenthesis beside a radicand (A) and alone in a radical (B, `dual_of_absent`), its alttext still `\mathstrut`.
/// Distinct from the guards above: a macro reaching the phantom through `\lx@mathstrut`, not `\vphantom` itself. Red
/// (PHANTOM round 3): an empty `XMHint name="mathstrut"`, `<msqrt><mi>a</mi></msqrt>`.
#[test]
fn mathstrut_is_a_vphantom_of_a_parenthesis() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-phantom/mathstrut_is_a_vphantom_of_a_parenthesis.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for id in ["p1.m1", "p2.m1"] {
    let expected = match id {
      "p1.m1" => {
        r#"<Math mode="inline" tex="\sqrt{\mathstrut a}\sqrt{\mathstrut b}" text="square-root@(a) * square-root@(b)" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp xml:id="p1.m1.1.1.2"><XMTok meaning="square-root"/><XMArg xml:id="p1.m1.1.1.2.2"><XMHint depth="2.5pt" height="7.5pt" name="vphantom" xml:id="p1.m1.1.1.2.2.1"><XMTok role="OPEN" stretchy="false">(</XMTok></XMHint><XMTok font="italic" role="UNKNOWN">a</XMTok></XMArg></XMApp><XMApp xml:id="p1.m1.1.1.3"><XMTok meaning="square-root"/><XMArg xml:id="p1.m1.1.1.3.2"><XMHint depth="2.5pt" height="7.5pt" name="vphantom" xml:id="p1.m1.1.1.3.2.1"><XMTok role="OPEN" stretchy="false">(</XMTok></XMHint><XMTok font="italic" role="UNKNOWN">b</XMTok></XMArg></XMApp></XMApp></XMath></Math>"#
      },
      _ => {
        r#"<Math mode="inline" tex="\sqrt{\mathstrut}" text="square-root@(absent)" xml:id="p2.m1"><XMath xml:id="p2.m1.1"><XMApp xml:id="p2.m1.1.1"><XMTok meaning="square-root"/><XMDual xml:id="p2.m1.1.1.2"><XMTok meaning="absent"/><XMHint depth="2.5pt" height="7.5pt" name="vphantom" xml:id="p2.m1.1.1.2.2"><XMTok role="OPEN" stretchy="false">(</XMTok></XMHint></XMDual></XMApp></XMath></Math>"#
      },
    };
    assert_element(&xml, "Math", &[&format!("xml:id=\"{id}\"")], expected);
  }
}
