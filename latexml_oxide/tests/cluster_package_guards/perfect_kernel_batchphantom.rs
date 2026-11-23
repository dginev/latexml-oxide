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

/// The core: content keeps ignoring the phantom (the XMDual's content branch is `absent`, or the item it stands by),
/// the presentation branch holds the hint with its parsed argument.
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
    r#"<Math mode="inline" tex="\sqrt{a\phantom{x}}" text="square-root@(a)" xml:id="p4.m1"><XMath xml:id="p4.m1.2"><XMApp xml:id="p4.m1.2.1"><XMTok meaning="square-root"/><XMDual xml:id="p4.m1.2.1.2"><XMRef idref="p4.m1.1" xml:id="p4.m1.2.1.2.1"/><XMWrap xml:id="p4.m1.2.1.2.2"><XMTok font="italic" role="UNKNOWN" xml:id="p4.m1.1">a</XMTok><XMHint depth="0.0pt" height="4.3pt" name="phantom" width="5.7pt" xml:id="p4.m1.2.1.2.2.1"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMHint></XMWrap></XMDual></XMApp></XMath></Math>"#,
  );
}
