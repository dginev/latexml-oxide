//! Red/green guards for batch 89d, two math-font follow-ups of 89c (OXIDIZED_DESIGN_DIVERGENCES #27, #914): an accent
//! name LaTeXML's own package bindings define, and agree on, decodes by its name in a declared font with no font map,
//! as a kernel accent's does since 89c (`reference_math_accent`, generated); and bbold's blackboard map is its own
//! family's (`U_blackboard`, `U_bbold`), not the whole `U` encoding's, so the other `U` fonts beside bbold keep their
//! own maps or none.
//!
//! One guard per path: the package-accent rows of `reference_math_accent` (sect08.rs `reference_accent`), and the
//! family-keyed bbold map (bbold_sty.rs; font.rs `decoding_map`, mathchar.rs `decode_in_font`, sect13.rs
//! `nfss_selected_font`), each in process and from the binary, eager and `--streaming`; each repro is under
//! `tools/perfect_kernel/repros/fonts-nfss/`, pdflatex-checked.
use super::perfect_kernel_batch89c::{check_everywhere, squash};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/",
      $path
    ))
  };
}

/// fonts-nfss/package_accent_decodes_by_its_name: accent names LaTeXML's package bindings define, redeclared at slots
/// of fonts with no map — a paper's copy of mathabx.dcl (`\ring` in `U/mathb` "38, `\widecheck` `\widebar`
/// `\widearrow` `\wideparen` in `U/mathx` "71/"73-"75; 2607.15091's 88 `\widecheck`) and raw newtxsf's `\dddot`
/// `\ddddot` `\widearc` in `LMS/ntxsy` 168/200/175 — are each the token the bindings agree on (mathabx's and
/// libertinust1math's `\widecheck` ˇ, fourier's and mathabx's `\wideparen` ⏜), as rendered. A name no binding defines
/// (`\myhat`), an under accent (`\utilde`: `\DeclareMathAccent` sets its accent above, tex.web §738) and a name the
/// bindings disagree on (`\overleftrightarrow`, stretchy in amsmath's, not in mathabx's) stay U+FFFD, `U/mathx`
/// saying so once. 89c printed all eleven U+FFFD with 3 Warnings.
#[test]
fn package_accent_decodes_by_its_name() {
  let fffd = |name: &str| format!("<XMTok name=\"{name}\" role=\"OVERACCENT\">\u{FFFD}</XMTok>");
  let expected_tokens = [
    fffd("myhat"),
    "<XMTok name=\"ring\" role=\"OVERACCENT\">\u{030A}</XMTok>".to_string(),
    r#"<XMTok name="widecheck" role="OVERACCENT">ˇ</XMTok>"#.to_string(),
    r#"<XMTok name="widebar" role="OVERACCENT">¯</XMTok>"#.to_string(),
    r#"<XMTok name="widearrow" role="OVERACCENT">→</XMTok>"#.to_string(),
    r#"<XMTok name="wideparen" role="OVERACCENT">⏜</XMTok>"#.to_string(),
    r#"<XMTok name="dddot" role="OVERACCENT">˙˙˙</XMTok>"#.to_string(),
    r#"<XMTok name="ddddot" role="OVERACCENT">˙˙˙˙</XMTok>"#.to_string(),
    r#"<XMTok name="widearc" role="OVERACCENT">⏜</XMTok>"#.to_string(),
    fffd("utilde"),
    fffd("overleftrightarrow"),
  ];
  let expected_tokens: Vec<&str> = expected_tokens.iter().map(String::as_str).collect();
  check_everywhere(
    repro!("package_accent_decodes_by_its_name.tex"),
    &[
      "Warning:missing_font_encoding:U/mathx Symbol font mathx (U/mathx) has no font map: the math accent \\myhat \
       (slot \"70) prints U+FFFD, as will any other symbol of this font it cannot decode",
    ],
    &expected_tokens,
  );
}

/// fonts-nfss/bbold_map_is_its_own_familys: beside `\usepackage{bbold}`, the other `U` fonts are their own — a
/// paper's mathx `\widehat` and `\widecheck` (mathabx.dcl:944-945) decode by their names (^, ˇ), stmaryrd's
/// `\boxslash` in `U/stmry` "1B (stmaryrd.sty:117) is U+FFFD with its font's Warning, a raw `U/msa` `\Cap` is ⋒
/// (`U_msa`) and a raw `U/eur` `\powersetsymbol` ℘ (`U_eur`) — while bbold's own font decodes through its map wherever
/// it is named: `\mathbb{R1}` (math characters, `decode_in_font`), `\textbb`, `\bbfamily`, `\usefont{U}{bbold}{m}{n}`
/// (text, `\fontencoding{U}` taking back its name for a family with a map) and `\DeclareSymbolFont{…}{U}{bbold}`
/// (`U_bbold`). With bbold's map registered for all of `U` (Perl bbold.sty.ltxml:30; 89c), they were 𝕡, 𝕢, σ, 𝕖 and
/// – — bbold's glyphs at those slots — with no Warning (review 89c r3 P1).
#[test]
fn bbold_map_is_its_own_familys() {
  let xml = squash(&check_everywhere(
    repro!("bbold_map_is_its_own_familys.tex"),
    &[
      "Warning:missing_font_encoding:U/stmry Symbol font stmry (U/stmry) has no font map: the math symbol \\boxslash \
       (slot \"1B) prints U+FFFD, as will any other symbol of this font it cannot decode",
    ],
    &[
      r#"<XMTok name="widehat" role="OVERACCENT">^</XMTok>"#,
      r#"<XMTok name="widecheck" role="OVERACCENT">ˇ</XMTok>"#,
      "<XMTok name=\"boxslash\" role=\"BINOP\">\u{FFFD}</XMTok>",
      r#"<XMTok name="Cap" role="BINOP">⋒</XMTok>"#,
      r#"<XMTok name="powersetsymbol" role="ID">℘</XMTok>"#,
      r#"<XMTok name="bbone" role="ID">𝟙</XMTok>"#,
    ],
  ));
  assert!(
    xml.contains(
      "<Math mode=\"inline\" tex=\"\\mathbb{R1}\" text=\"ℝ * 𝟙\"><XMath><XMApp>\
       <XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok role=\"UNKNOWN\">ℝ</XMTok>\
       <XMTok font=\"blackboard\" role=\"UNKNOWN\">𝟙</XMTok></XMApp></XMath></Math>"
    ),
    "{xml}"
  );
  let text = "<text font=\"blackboard\">ℝ𝟙</text>";
  assert_eq!(xml.matches(text).count(), 3, "{xml}");
}
