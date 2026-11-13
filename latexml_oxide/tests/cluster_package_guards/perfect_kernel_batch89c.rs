//! Red/green guards for batch 89c, two math-font follow-ups of 89b (OXIDIZED_DESIGN_DIVERGENCES #27): a kernel math
//! accent a package redeclares at another slot of a font with no font map is the token LaTeXML's own binding makes of
//! its name (the name is the accent's meaning), and the Euler Roman family `U/eur` decodes through the math italic
//! layout (OML) at the 111 slots eurm10 shares with cmmi10, the 17 it lacks staying undecodable — gaps of the font,
//! where no kernel name decodes either (round 2), as at msbm's five (round 3).
//!
//! One guard per path: `\DeclareMathAccent` in a font with no map (sect08.rs `reference_accent`), the `U_eur` family
//! map and its gaps (`u_fontmap.rs`, `symbol_font_lacks_slot`, for an accent and for a symbol), and a mapped font's
//! empty slot that is no gap (msam/msbm glyphs the maps have no single character for), where a reference name still
//! decodes (`reference_symbol`, OXIDIZED_DESIGN_DIVERGENCES #910). Each runs in process and from the binary, eager
//! and `--streaming`; each repro is under `tools/perfect_kernel/repros/fonts-nfss/`, pdflatex-checked.
use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  perfect_kernel_batch89a::{cli_conversion, named_tokens},
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/",
      $path
    ))
  };
}

/// The `Warning:` lines of a log, each its first line, in order.
fn warnings(log: &str) -> Vec<String> {
  log
    .lines()
    .filter(|line| line.starts_with("Warning:"))
    .map(ToString::to_string)
    .collect()
}

/// `xml` with each line's indentation dropped, the lines joined and the `xml:id`s dropped.
pub(super) fn squash(xml: &str) -> String {
  let mut out = String::new();
  for line in xml.lines() {
    let mut rest = line.trim();
    while let Some(start) = rest.find(" xml:id=\"") {
      out.push_str(&rest[..start]);
      let tail = &rest[start + " xml:id=\"".len()..];
      rest = tail.find('"').map_or("", |end| &tail[end + 1..]);
    }
    out.push_str(rest);
  }
  out
}

/// Checks a conversion's Errors, Warnings and named tokens, in process and from the binary, eager and `--streaming`;
/// returns the in-process XML.
#[track_caller]
pub(super) fn check_everywhere(
  tex: &str,
  expected_warnings: &[&str],
  expected_tokens: &[&str],
) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, expected_warnings.len()),
    "{log}"
  );
  assert_eq!(warnings(&log), expected_warnings, "{log}");
  assert_eq!(named_tokens(&xml), expected_tokens, "{xml}");
  for streaming in [false, true] {
    let (log, cli_xml) = cli_conversion(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, expected_warnings.len()),
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      warnings(&log),
      expected_warnings,
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      named_tokens(&cli_xml),
      expected_tokens,
      "streaming={streaming}: {cli_xml}"
    );
  }
  xml
}

/// fonts-nfss/kernel_accent_decodes_by_its_name: newtxsf's eleven kernel accents, redeclared in `lettersA`
/// (`U/ntxsfmia`) and `symbols` (`LMS/ntxsy`) at slots of their own (newtxsf.sty:301-311, :1332; 2106.08363,
/// 1808.01384, 2406.11942, 1902.03183), and a paper's `\widehat` in `U/mathx` at "70 (2607.15091): each the token the
/// kernel binding makes of its name — glyph, `name`, stretchiness, as `$\hat{x}$` without the package — over its base,
/// nested, in a script, over an empty base, and in 1902.03183's align cell, kept whole. mathabx's own `\widecheck`,
/// no kernel name, is the token LaTeXML's mathabx and libertinust1math bindings agree on, ˇ, with no Warning since 89d
/// (`perfect_kernel_batch89d::package_accent_decodes_by_its_name`; U+FFFD and its font's one Warning in 89c). 89b
/// printed U+FFFD for each with 3 Warnings; 89a an empty
/// token (MathML `<mo>hat</mo>`) and dropped the cell. A kernel accent in a symbol font never declared (pdflatex:
/// "Symbol font `mathx' is not defined", an in-process input) stays U+FFFD and is said so.
#[test]
fn kernel_accent_decodes_by_its_name() {
  let tex = repro!("kernel_accent_decodes_by_its_name.tex");
  let expected_warnings: [&str; 0] = [];
  let hat = r#"<XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok>"#;
  let bar = r#"<XMTok name="bar" role="OVERACCENT" stretchy="false">¯</XMTok>"#;
  let script_hat =
    r#"<XMTok fontsize="70%" name="hat" role="OVERACCENT" stretchy="false">^</XMTok>"#;
  let zero = r#"<XMTok fontsize="70%" meaning="0" name="0" role="NUMBER">0</XMTok>"#;
  let expected_tokens = [
    r#"<XMTok name="acute" role="OVERACCENT">´</XMTok>"#,
    r#"<XMTok name="grave" role="OVERACCENT">`</XMTok>"#,
    r#"<XMTok name="ddot" role="OVERACCENT">¨</XMTok>"#,
    r#"<XMTok name="tilde" role="OVERACCENT" stretchy="false">~</XMTok>"#,
    bar,
    r#"<XMTok name="breve" role="OVERACCENT">˘</XMTok>"#,
    r#"<XMTok name="check" role="OVERACCENT" stretchy="false">ˇ</XMTok>"#,
    hat,
    r#"<XMTok name="vec" role="OVERACCENT" stretchy="false">→</XMTok>"#,
    r#"<XMTok name="dot" role="OVERACCENT">˙</XMTok>"#,
    "<XMTok name=\"mathring\" role=\"OVERACCENT\">\u{030A}</XMTok>",
    r#"<XMTok meaning="plus" name="+" role="ADDOP">+</XMTok>"#,
    r#"<XMTok name="widehat" role="OVERACCENT">^</XMTok>"#,
    r#"<XMTok name="widecheck" role="OVERACCENT">ˇ</XMTok>"#,
    hat,
    bar,
    script_hat,
    hat,
    bar,
    zero,
    bar,
    zero,
  ];
  let xml = squash(&check_everywhere(tex, &expected_warnings, &expected_tokens));
  let x = r#"<XMTok font="italic" role="UNKNOWN">x</XMTok>"#;
  // Nested: the hat over the barred x.
  assert!(
    xml.contains(&format!("<XMApp>{hat}<XMApp>{bar}{x}</XMApp></XMApp>")),
    "{xml}"
  );
  // The align cell, whole: its bar over `u`.
  assert!(
    xml.contains(&format!(
      "<td align=\"right\"><Math mode=\"inline\" tex=\"\\displaystyle\\bar{{u}}\" text=\"bar@(u)\"><XMath><XMApp>\
       {bar}<XMTok font=\"italic\" role=\"UNKNOWN\">u</XMTok></XMApp></XMath></Math></td>"
    )),
    "{xml}"
  );
  let (undeclared, undeclared_xml) = convert_with(
    "\\documentclass{article}\n\\DeclareMathAccent{\\hat}{\\mathalpha}{mathx}{\"70}\n\\begin{document}\n\
     $\\hat{x}$\n\\end{document}\n",
    Some(PRELOAD),
  );
  assert_eq!(
    warnings(&undeclared),
    [
      "Warning:missing_font_encoding:\\symmathx Symbol font mathx is not declared: the math accent \\hat (slot \"70) \
       prints U+FFFD, as will any other symbol of this font it cannot decode"
    ],
    "{undeclared}"
  );
  assert_eq!(error_count(&undeclared), 0, "{undeclared}");
  assert_eq!(
    named_tokens(&undeclared_xml),
    ["<XMTok name=\"hat\" role=\"OVERACCENT\">\u{FFFD}</XMTok>"],
    "{undeclared_xml}"
  );
}

/// fonts-nfss/euler_roman_decodes_as_math_italic: program.sty's `\powersetsymbol` in `U/eur` at "7D (program.sty:834-838,
/// older sn-jnl.cls copies) is ℘, and the font's ℓ at "60 decodes, through the OML layout eurm10 shares with cmmi10
/// ("TEX MATH ITALIC SUBSET"). The slots eurm10 lacks (pdflatex: "Missing character") are gaps of the font: the
/// kernel's `\star` at "3F (fontmath.ltx's `symbols` slot, the 89a reference path) and `\vec` at "7E (the accent
/// path), as well as the flat at "5B, are U+FFFD, each symbol font saying "has no character at slot" once. 89c round 1
/// printed ⋆ and → by their names with no Warning (review B1, P1; int120a ⋆ alike); 89b printed U+FFFD for all, its
/// Warning saying the font had no map. The other mapped family with gaps, msbm (`U/msb`; "5F, "62-"65 by tftopl): AMS
/// names declared in `AMSb` at their `AMSa` slots — `\Cap` "65, `\Subset` "62, `\varpropto` "5F (amssymb.sty), where
/// msbm draws nothing (pdflatex: "Missing character … in font msbm10") — are U+FFFD, `AMSb` saying so once. Through
/// 89c round 2 they were ⋒, ⋐ and ∝ by their names with no Warning (review 89c r2 S1; int120a alike). Both families'
/// gaps are generated (`U_FONT_GAPS`, u_fontmap.rs); msam has none.
#[test]
fn euler_roman_decodes_as_math_italic() {
  let tex = repro!("euler_roman_decodes_as_math_italic.tex");
  let expected_warnings = [
    "Warning:missing_font_encoding:U/eur Symbol font eulerletters (U/eur) has no character at slot \"3F: the math \
     symbol \\star (slot \"3F) prints U+FFFD, as will any other symbol of this font it cannot decode",
    "Warning:missing_font_encoding:U/eur Symbol font eu (U/eur) has no character at slot \"7E: the math accent \\vec \
     (slot \"7E) prints U+FFFD, as will any other symbol of this font it cannot decode",
    "Warning:missing_font_encoding:U/msb Symbol font AMSb (U/msb) has no character at slot \"65: the math symbol \
     \\Cap (slot \"65) prints U+FFFD, as will any other symbol of this font it cannot decode",
  ];
  let expected_tokens = [
    r#"<XMTok meaning="subset-of-or-equals" name="subseteq" role="RELOP">⊆</XMTok>"#,
    r#"<XMTok fontsize="144%" name="powersetsymbol" role="ID" yoffset="2.2pt">℘</XMTok>"#,
    r#"<XMTok name="euell" role="ID">ℓ</XMTok>"#,
    "<XMTok name=\"star\" role=\"BINOP\">\u{FFFD}</XMTok>",
    "<XMTok name=\"euflat\" role=\"ID\">\u{FFFD}</XMTok>",
    "<XMTok name=\"vec\" role=\"OVERACCENT\">\u{FFFD}</XMTok>",
    "<XMTok name=\"Cap\" role=\"BINOP\">\u{FFFD}</XMTok>",
    "<XMTok name=\"Subset\" role=\"RELOP\">\u{FFFD}</XMTok>",
    "<XMTok name=\"varpropto\" role=\"RELOP\">\u{FFFD}</XMTok>",
  ];
  check_everywhere(tex, &expected_warnings, &expected_tokens);
}

/// fonts-nfss/mapped_font_glyph_without_a_character_decodes_by_name: oz.sty's and zed-csp's raw `\nleqslant`,
/// `\ngeqq` and `\nsubseteqq` in `AMSb` (`U/msb`, as amsfonts declares it) at "0A/"15/"22 and amsfonts.sty's
/// `\dabar@` in `AMSa` (`U/msa`) at "39 — their amssymb slots, where the msbm/msam maps are empty because the glyph has
/// no single character (a negated pair, a dash bar), not because the font lacks it — are the tokens LaTeXML's own
/// amssymb binding makes of the names, with 0 Warnings. Distinct from the eur gaps: the empty slot of a mapped font is
/// a gap only where the font records it (`symbol_font_lacks_slot`); the no-map rule alone printed these U+FFFD.
#[test]
fn mapped_font_glyph_without_a_character_decodes_by_name() {
  let dabar = r#"<XMTok name="dabar@" role="ID">╌</XMTok>"#;
  check_everywhere(
    repro!("mapped_font_glyph_without_a_character_decodes_by_name.tex"),
    &[],
    &[
      r#"<XMTok meaning="not-less-than-nor-equals" name="nleqslant" role="RELOP">⩽̸</XMTok>"#,
      r#"<XMTok meaning="not-greater-than-nor-equals" name="ngeqq" role="RELOP">≧̸</XMTok>"#,
      r#"<XMTok meaning="not-subset-of-nor-equals" name="nsubseteqq" role="RELOP">⫅̸</XMTok>"#,
      dabar,
      dabar,
    ],
  );
}
