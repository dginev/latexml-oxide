//! Guards for NOTCANCEL: math_common's `\not` rewrite puts its operand back by moving it, as Perl's `appendChild`
//! does (math_common.pool.ltxml:341 the cancel path, :354 the token path), and gives the cancel `XMApp` the `\not`'s
//! box (Perl `_box => $box`). A copy left every `\not` nested in the operand outside the tree for the pass that had
//! already selected it (revPERFF). Every expected `XMath` is Perl 0.8.8's on the same host, ids aside.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const D4: &str = r#"<XMath><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok meaning="not-equals" name="not-=" role="RELOP">≠</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMath>"#;
const NOT_NOT: &str =
  r#"<XMath><XMTok meaning="not-equals" name="not-=" role="RELOP">≠</XMTok></XMath>"#;

/// Each formula's `tex` and its `XMath`, the `xml:id`s and the whitespace between elements dropped.
fn xmaths(xml: &str) -> Vec<(String, String)> {
  let math =
    regex::Regex::new(r#"(?s)<Math [^>]*?tex="([^"]*)"[^>]*>.*?(<XMath>.*?</XMath>)"#).unwrap();
  let id = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let gap = regex::Regex::new(r">\s+<").unwrap();
  let xml = id.replace_all(xml, "");
  math
    .captures_iter(&xml)
    .map(|c| (c[1].to_string(), gap.replace_all(&c[2], "><").into_owned()))
    .collect()
}

fn assert_xmaths(xml: &str, expected: &[(&str, &str)]) {
  let got = xmaths(
    &regex::Regex::new(r#" idref="[^"]*""#)
      .unwrap()
      .replace_all(xml, ""),
  );
  let expected: Vec<(String, String)> = expected
    .iter()
    .map(|(t, x)| ((*t).to_string(), (*x).to_string()))
    .collect();
  assert_eq!(got, expected, "{xml}");
}

/// The element carrying `xml:id="{id}"`, whole, its ids and the whitespace between its elements dropped.
fn element_by_id(xml: &str, id: &str) -> String {
  let start = xml[..xml.find(&format!(r#" xml:id="{id}""#)).expect(id)]
    .rfind('<')
    .unwrap();
  let tag = regex::Regex::new(r"<(/?)[A-Za-z]+[^>]*?(/?)>").unwrap();
  let mut depth = 0;
  for m in tag.captures_iter(&xml[start..]) {
    let whole = m.get(0).unwrap();
    match (&m[1], &m[2]) {
      ("/", _) => depth -= 1,
      (_, "/") => {},
      _ => depth += 1,
    }
    if depth == 0 {
      let element = &xml[start..start + whole.end()];
      let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
      return regex::Regex::new(r">\s+<")
        .unwrap()
        .replace_all(&ids.replace_all(element, ""), "><")
        .into_owned();
    }
  }
  panic!("unclosed element {id}")
}

/// The cancel path nests: a `\not` inside a struck-out root, fraction or script is rewritten in the same pass, at
/// any depth (W1 depth 3, D4), and the moved operand keeps its size (W2: 70%, a copy re-scaled it to 49%); the token
/// path moves too, so `\not\not=` ends as Perl's `≠`; XMRefs into the moved operand (`\binom`'s) keep their targets.
#[test]
fn not_cancel_moves_its_operand() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/not_cancel_moves_its_operand.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_xmaths(&xml, &[
    (
      r"\not\sqrt{\not\sqrt{a\not=b}}",
      r#"<XMath><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok meaning="not-equals" name="not-=" role="RELOP">≠</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp></XMath>"#,
    ),
    (
      r"\not\frac{\not\sqrt{a\not=b}}{c}",
      r#"<XMath><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok mathstyle="text" meaning="divide" role="FRACOP"/><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok fontsize="70%" meaning="not-equals" name="not-=" role="RELOP">≠</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">b</XMTok></XMApp></XMApp></XMApp><XMTok font="italic" fontsize="70%" role="UNKNOWN">c</XMTok></XMApp></XMApp></XMath>"#,
    ),
    (r"\not\sqrt{\not\sqrt{\not\sqrt{a\not=b}}}", D4),
    (
      r"x\not\in A",
      r#"<XMath><XMApp><XMTok meaning="not-element-of" name="not-in" role="RELOP">∉</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="italic" role="UNKNOWN">A</XMTok></XMApp></XMath>"#,
    ),
    (
      r"\not\frac{1}{2}",
      r#"<XMath><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok mathstyle="text" meaning="divide" role="FRACOP"/><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath>"#,
    ),
    (
      r"y_{\not\frac{a}{\not\sqrt{b}}}^{\not\sqrt{\not\sqrt{c\not\subset d}}}",
      r#"<XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">y</XMTok><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok mathstyle="script" meaning="divide" role="FRACOP"/><XMTok font="italic" fontsize="50%" role="UNKNOWN">a</XMTok><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMTok font="italic" fontsize="50%" role="UNKNOWN">b</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok fontsize="70%" meaning="not-subset-of" name="not-subset" role="RELOP">⊄</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">c</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">d</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMath>"#,
    ),
    (
      r"\not\frac{A\not\subset B}{\not\sqrt{C\not\equiv D}}",
      r#"<XMath><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok mathstyle="text" meaning="divide" role="FRACOP"/><XMApp><XMTok fontsize="70%" meaning="not-subset-of" name="not-subset" role="RELOP">⊄</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">A</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">B</XMTok></XMApp><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok fontsize="70%" meaning="not-equivalent-to" name="not-equiv" role="RELOP">≢</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">C</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">D</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp></XMath>"#,
    ),
    (r"\not\not=", NOT_NOT),
    (
      r"\not\binom{n}{\not\sqrt{k\not\in K}}",
      r#"<XMath><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMDual><XMApp><XMTok meaning="binomial" name="binom" role="UNKNOWN"/><XMRef/><XMRef/></XMApp><XMWrap><XMTok role="OPEN" stretchy="true">(</XMTok><XMApp><XMTok mathstyle="text" role="FRACOP" thickness="0pt"/><XMTok font="italic" fontsize="70%" role="UNKNOWN">n</XMTok><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok fontsize="70%" meaning="not-element-of" name="not-in" role="RELOP">∉</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">k</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">K</XMTok></XMApp></XMApp></XMApp></XMApp><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMApp></XMath>"#,
    ),
  ]);
  // The binom's XMRefs point into the moved operand: they still name its `n` and its rewritten `\not\sqrt`.
  let binom = &xml[xml.find(r#"tex="\not\binom"#).unwrap()..];
  let binom = &binom[..binom.find("</Math>").unwrap()];
  let targets: Vec<String> = regex::Regex::new(r#"idref="([^"]*)""#)
    .unwrap()
    .captures_iter(binom)
    .map(|c| element_by_id(&xml, &c[1]))
    .collect();
  assert_eq!(targets, [
    r#"<XMTok font="italic" fontsize="70%" role="UNKNOWN">n</XMTok>"#,
    r#"<XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok fontsize="70%" meaning="not-element-of" name="not-in" role="RELOP">∉</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">k</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">K</XMTok></XMApp></XMApp></XMApp>"#,
  ]);
}

/// Plain TeX registers the rule once (LaTeX twice: latex_constructs reloads math_common, as Perl's does), so one pass
/// alone rewrites every level here; D4 and `\not\not=` give the LaTeX run's `XMath`, so its second pass finds no
/// `\not` left to match.
#[test]
fn not_rewrite_is_one_pass_plain() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/not_rewrite_is_one_pass_plain.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_xmaths(&xml, &[
    (
      r"\not\sqrt{a\not=b}",
      r#"<XMath><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok meaning="not-equals" name="not-=" role="RELOP">≠</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMApp></XMApp></XMath>"#,
    ),
    (r"\not\sqrt{\not\sqrt{\not\sqrt{a\not=b}}}", D4),
    (r"\not\not=", NOT_NOT),
    (
      r"x_{\not\sqrt{y\not\in A}}",
      r#"<XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMApp><XMTok enclose="updiagonalstrike" meaning="not" role="ENCLOSE"/><XMApp><XMTok meaning="square-root"/><XMApp><XMTok fontsize="70%" meaning="not-element-of" name="not-in" role="RELOP">∉</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">y</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">A</XMTok></XMApp></XMApp></XMApp></XMApp></XMath>"#,
    ),
  ]);
}
