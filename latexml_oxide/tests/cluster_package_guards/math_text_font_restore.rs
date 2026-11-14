//! Entering text mode restores the pre-math text font only when leaving math
//! (Perl Stomach.pm:514-535, batch 56ja): a font switch inside a `\vbox`
//! inside math survives the `\hbox` it wraps. Repro
//! `tools/perfect_kernel/repros/fonts-nfss/math_box_keeps_text_font.tex`
//! (cascade-french, fancyvrb `BVerbatim[baseline=c]`). The controls pin math
//! entry itself (a script) and the restore where it belongs: a text box
//! directly in math, and an `\hbox` in a math-level `\noalign`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

#[test]
fn text_font_survives_an_hbox_inside_math() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/math_box_keeps_text_font.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "inline-block",
    &[],
    r#"<inline-block class="ltx_markedasmath" vattach="bottom"><p><text font="typewriter">\foo\{x\}</text></p></inline-block>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="x_{1}""#],
    r#"<Math mode="inline" tex="x_{1}" text="x _ 1" xml:id="p1.m2"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "text",
    &[r#"class="ltx_markedasmath""#],
    r#"<text class="ltx_markedasmath" font="bold">a</text>"#,
  );
  assert_element(
    &xml,
    "XMText",
    &[],
    r#"<XMText><text font="bold">where</text></XMText>"#,
  );
}
