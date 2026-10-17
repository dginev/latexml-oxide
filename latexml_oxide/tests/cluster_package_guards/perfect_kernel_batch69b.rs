//! Red/green guards for batch 69b: low-risk shapes from the rc_struct3 study (agent_reports/2026-10-17_rc_struct3.md).
//! Item 1 (B1) — bm.sty's `\bm` leaves a script that ends its argument where it stands, so it takes its argument
//! after `\bm` (bm.sty:162-217 `\bm@general`). Item 2 (L1) — the kernel's `\@trivlist` is locked as in Perl
//! (latex_constructs.pool.ltxml:1673), so an ACM 1.8 class's spacing-only redefinition keeps `\list` a list.
//! Item 3 (L2) — `\endlist` with no list open does nothing, as latex.ltx:15915-15930 `\endtrivlist`. Item 4 (E2) —
//! mdwtab's `array` enters math itself (mdwtab.sty:505-510), so in text it is a formula without "Missing $"; Wooding's
//! mathenv loads mdwtab (mathenv.sty:53).
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

/// list-structure/class_redefining_trivlist_keeps_the_list: vldb.cls:1118's `\def\@trivlist{…}` (spacing only) before a
/// `{list}` (pdflatex 0 errors: one bulleted item). Before 69b: 1 Error "Attempt to end mode internal_vertical" (Perl
/// 0, its `\@trivlist` locked; 1212.6177, 1710.00867).
#[test]
fn class_redefining_trivlist_keeps_the_list() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/class_redefining_trivlist_keeps_the_list.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "itemize",
    &[],
    r#"<itemize><item xml:id="S0.I1.i1"><tags><tag><Math mode="inline" tex="\bullet" text="bullet" xml:id="S0.I1.i1.m1"><XMath><XMTok name="bullet" role="MULOP">∙</XMTok></XMath></Math></tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>One.</p></para></item></itemize>"#,
  );
}

/// list-structure/endlist_with_no_list_open_does_nothing: a load-time `{…\endlist}` (siuro210301.cls:995-1000) and a
/// proof whose end code calls `\endlist` (0710.0460), then a real `{itemize}` (pdflatex 0 errors). Before 69b: 2
/// Errors "Attempt to end mode internal_vertical" (Perl the same).
#[test]
fn endlist_with_no_list_open_does_nothing() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/endlist_with_no_list_open_does_nothing.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p2\""],
    r#"<para xml:id="p2"><p><text font="bold">Proof:</text>It holds.QED</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p3\""],
    r#"<para xml:id="p3"><p>After.</p><itemize xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>A list still closes.</p></para></item></itemize></para>"#,
  );
}

/// alignment-bindings/mdwtab_array_in_text_is_a_formula: `\begin{center}\begin{array}{rl}…\end{array}\end{center}`
/// under mdwtab, and the same under Wooding's mathenv (pdflatex 0 errors: a centred 2x2 array). Before 69b: 1 Error
/// "Missing $ inserted" each (the kernel's `[c]` array check).
#[test]
fn mdwtab_array_in_text_is_a_formula() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/mdwtab_array_in_text_is_a_formula.tex"
  );
  for tex in [tex.to_string(), tex.replace("{mdwtab}", "{mathenv}")] {
    let (stderr, xml) = convert_with(&tex, None);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(warning_count(&stderr), 0, "{stderr}");
    assert_element(
      &xml,
      "p",
      &["align=\"center\""],
      r#"<p align="center"><Math mode="inline" tex="\begin{array}[]{rl}a&amp;b\\&#10;c&amp;d\end{array}" text="Array[[a, b], [c, d]]" xml:id="m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="right"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell><XMCell align="left"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell></XMRow><XMRow><XMCell align="right"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMCell><XMCell align="left"><XMTok font="italic" role="UNKNOWN">d</XMTok></XMCell></XMRow></XMArray></XMath></Math></p>"#,
    );
  }
}
