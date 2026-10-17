//! Red/green guards for batch 69b: low-risk shapes from the rc_struct3 study (agent_reports/2026-10-17_rc_struct3.md).
//! Item 1 (B1) — bm.sty's `\bm` leaves a script that ends its argument where it stands, so it takes its argument
//! after `\bm` (bm.sty:162-217 `\bm@general`).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

/// package-bindings/bm_trailing_script_takes_the_next_argument: `\bm{G_}{(10)}`, `M\bm_{max}`, `\bm{D^}{t}_p`
/// (pdflatex 0 errors: a bold G subscripted by (10), `M_{max}`, a bold D with ^t and _p). Before 69b: 3 Errors
/// "Missing sub/superscript argument" (Perl the same, bm.sty.ltxml:21).
#[test]
fn bm_trailing_script_takes_the_next_argument() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/bm_trailing_script_takes_the_next_argument.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m1\""],
    r#"<Math mode="inline" tex="\bm{G}_{(10)}" text="G _ 10" xml:id="p1.m1"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="bold italic" role="UNKNOWN">G</XMTok><XMDual><XMRef idref="p1.m1.1"/><XMWrap><XMTok fontsize="70%" role="OPEN" stretchy="false">(</XMTok><XMTok fontsize="70%" meaning="10" role="NUMBER" xml:id="p1.m1.1">10</XMTok><XMTok fontsize="70%" role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m2\""],
    r#"<Math mode="inline" tex="M_{max}" text="M _ (m * a * x)" xml:id="p1.m2"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">M</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">m</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m3\""],
    r#"<Math mode="inline" tex="\bm{D}^{t}_{p}" text="(D ^ t) _ p" xml:id="p1.m3"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="bold italic" role="UNKNOWN">D</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">t</XMTok></XMApp><XMTok font="italic" fontsize="70%" role="UNKNOWN">p</XMTok></XMApp></XMath></Math>"#,
  );
}
