//! Red/green guards for batch 89b: a math slot its symbol font cannot decode — no glyph in the declared font's map,
//! or no map at all, and no reference slot (89a) — is U+FFFD, never the raw slot as a character, and when such a symbol
//! is first printed, its symbol font says so in one `Warning:missing_font_encoding:<encoding>/<family>`
//! (OXIDIZED_DESIGN_DIVERGENCES #27, user ruling 2026-10-29). Was kpfonts' `\neq` as `,`, an Ω at "0A as a line break, a TS1 hole as `A`, mathabx's `\sum` as `°`.
//!
//! One guard per path: `\DeclareMathSymbol` (several symbol fonts, several undecodable slots in one, a font with a map
//! but a hole, decodable symbols beside them; also through the binary, eager and `--streaming`),
//! `\DeclareMathAccent` (an accent and a symbol of one font: one Warning; the accent's base kept, in an align cell
//! too), and where the Warning is raised: when the document absorbs (prints) a box holding the symbol, never where a
//! box is only digested (`latexml_core::document::ABSORB_WARNING_PROPERTY`). The pinned outputs of the guards this
//! changed are updated where they live (`perfect_kernel_batch68v`, `perfect_kernel_batch89a`,
//! `perfect_kernel_batch65l`). Each repro is under `tools/perfect_kernel/repros/fonts-nfss/`, pdflatex-checked.
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

/// `xml` with each line's indentation dropped, the lines joined and the `xml:id`s dropped: an element whole on one
/// line, the same in process and from the binary.
fn squash(xml: &str) -> String {
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

/// The `Warning:` lines of a log, each its first line, in order.
fn warnings(log: &str) -> Vec<String> {
  log
    .lines()
    .filter(|line| line.starts_with("Warning:"))
    .map(ToString::to_string)
    .collect()
}

/// The location line (`at …`) of each Warning of a log, in order: the line after its first.
fn warning_locations(log: &str) -> Vec<String> {
  let lines: Vec<&str> = log.lines().collect();
  lines
    .iter()
    .enumerate()
    .filter(|(_, line)| line.starts_with("Warning:"))
    .map(|(at, _)| {
      lines
        .get(at + 1)
        .map_or_else(String::new, |line| line.trim().to_string())
    })
    .collect()
}

/// fonts-nfss/undecodable_math_slot_is_fffd_and_warns: kpfonts' `\neq`, `\doteq`, `\nequiv` in `U/jkpsyc` (1604.04618,
/// 2008.13643), heppennames2's shape `U/fsy` with Ω at "0A and Λ at "03 (2307.11186), a TS1 slot with no glyph (`"41`):
/// each U+FFFD, and one Warning per symbol font naming its encoding, family and first printed symbol — three fonts, three
/// Warnings, however many slots each loses; `\nsim` and cmll's `\invamp` (2402.09138), declared and never used, say
/// nothing. `\tcmho` (TS1 "4D) and `\mysquare` (`U/msa` "03) decode as before, and
/// euler.sty's character symbols in `U/euf` print themselves and warn nothing: `(` `)` at their own codes with no name,
/// `"` at "7D keeping its `name` as 89a did (euler.sty:67-74; 89b round 1 dropped the name of newtxsf's digits).
/// 89a printed `,`, U+FFFD, `.`, a line break, U+FFFD and an italic `A`, with 0 warnings. The same in process and from
/// the binary, eager and `--streaming`.
#[test]
fn undecodable_math_slot_is_fffd_and_warns() {
  let tex = repro!("undecodable_math_slot_is_fffd_and_warns.tex");
  let expected_tokens = [
    "<XMTok name=\"neq\" role=\"RELOP\">\u{FFFD}</XMTok>",
    "<XMTok name=\"doteq\" role=\"RELOP\">\u{FFFD}</XMTok>",
    "<XMTok name=\"nequiv\" role=\"RELOP\">\u{FFFD}</XMTok>",
    "<XMTok name=\"Upomega\" role=\"ID\">\u{FFFD}</XMTok>",
    "<XMTok name=\"Uplambda\" role=\"ID\">\u{FFFD}</XMTok>",
    "<XMTok name=\"tcnone\" role=\"ID\">\u{FFFD}</XMTok>",
    "<XMTok name=\"tcmho\" role=\"ID\">\u{2127}</XMTok>",
    "<XMTok name=\"mysquare\" role=\"ID\">\u{25A1}</XMTok>",
    r#"<XMTok name="&quot;" role="ID">"</XMTok>"#,
  ];
  let expected_warnings = [
    "Warning:missing_font_encoding:U/jkpsyc Symbol font symbolsC (U/jkpsyc) has no font map: the math symbol \\neq \
     (slot \"2C) prints U+FFFD, as will any other symbol of this font it cannot decode",
    "Warning:missing_font_encoding:U/fsy Symbol font ugrfr (U/fsy) has no font map: the math symbol \\Upomega (slot \
     \"0A) prints U+FFFD, as will any other symbol of this font it cannot decode",
    "Warning:missing_font_encoding:TS1/cmr Symbol font textcomp (TS1/cmr) has no Unicode glyph at slot \"41: the math \
     symbol \\tcnone (slot \"41) prints U+FFFD, as will any other symbol of this font it cannot decode",
  ];
  let parens = [
    r#"<XMTok role="OPEN" stretchy="false">(</XMTok>"#,
    r#"<XMTok role="CLOSE" stretchy="false">)</XMTok>"#,
  ];
  // Each simple symbol's Warning is at its first print, the position its box carries (89b round 2 kept it).
  let expected_locations =
    ["line 40 col 7", "line 40 col 38", "line 40 col 59"].map(|at| format!("at t; {at} - {at}"));
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  assert_eq!(warnings(&log), expected_warnings, "{log}");
  assert_eq!(named_tokens(&xml), expected_tokens, "{xml}");
  assert!(parens.iter().all(|paren| xml.contains(paren)), "{xml}");
  assert!(!xml.contains("\n</XMTok>"), "{xml}");
  for streaming in [false, true] {
    let (log, xml) = cli_conversion(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 3),
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      warnings(&log),
      expected_warnings,
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      warning_locations(&log),
      expected_locations,
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      named_tokens(&xml),
      expected_tokens,
      "streaming={streaming}: {xml}"
    );
    assert!(
      parens.iter().all(|paren| xml.contains(paren)),
      "streaming={streaming}: {xml}"
    );
  }
}

/// fonts-nfss/undecodable_math_accent_is_fffd_and_warns: an accent of a paper's own name, `\mycheck`, at mathabx's
/// `\widecheck` slot, and mathabx's `\sum`, in `U/mathx` (mathabx.dcl:945, :748; 1806.02506, whose `\widecheck`
/// decodes by its name since 89d, `perfect_kernel_batch89d::package_accent_decodes_by_its_name`): both U+FFFD, and one
/// Warning for the symbol font they share, naming the math accent. The accent stays over its base, in an align cell too: 1902.03183's eq. 9 `\bar{u} &= a_0.` (newtxsf's
/// `lettersA` `\bar`) lost that cell whole in 89a, where the accent was empty, and only the accent's `name` reached
/// MathML (`<mo>hat</mo>`). 89a gave an empty accent and `°` (the raw slot "B0), with 0 warnings.
/// The Warning is at the accent (a constructor: its position recorded at digestion,
/// `latexml_core::document::ABSORB_WARNING_LOCATOR_PROPERTY`), from the binary eager and `--streaming`; 89b round 2
/// said `at Anonymous String` (newtxsf's `\bar`, `\dot` in 1902.03183 alike). A symbol font never declared
/// (`\symmathx` undefined: pdflatex stops on it, "LaTeX Error: Symbol font mathx is not defined", so an in-process
/// input, not a repro) is said so, naming the accent, at the accent.
#[test]
fn undecodable_math_accent_is_fffd_and_warns() {
  let tex = repro!("undecodable_math_accent_is_fffd_and_warns.tex");
  let expected_warnings = [
    "Warning:missing_font_encoding:U/mathx Symbol font mathx (U/mathx) has no font map: the math accent \\mycheck \
     (slot \"71) prints U+FFFD, as will any other symbol of this font it cannot decode",
  ];
  let accent_location = ["at t; line 22 col 10 - line 22 col 10"];
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(warnings(&log), expected_warnings, "{log}");
  assert_eq!(warning_locations(&log), accent_location, "{log}");
  for streaming in [false, true] {
    let (log, _) = cli_conversion(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 1),
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      warnings(&log),
      expected_warnings,
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      warning_locations(&log),
      accent_location,
      "streaming={streaming}: {log}"
    );
  }
  let (undeclared, _) = convert_with(
    "\\documentclass{article}\n\\DeclareMathAccent{\\widecheck}{\\mathalpha}{mathx}{\"71}\n\\begin{document}\n\
     $\\widecheck{x}$\n\\end{document}\n",
    Some(PRELOAD),
  );
  assert_eq!(
    (error_count(&undeclared), warning_count(&undeclared)),
    (0, 1),
    "{undeclared}"
  );
  assert_eq!(
    warnings(&undeclared),
    [
      "Warning:missing_font_encoding:\\symmathx Symbol font mathx is not declared: the math accent \\widecheck (slot \
       \"71) prints U+FFFD, as will any other symbol of this font it cannot decode"
    ],
    "{undeclared}"
  );
  assert_eq!(
    warning_locations(&undeclared),
    ["at t; line 4 col 12 - line 4 col 12"],
    "{undeclared}"
  );
  let xml = squash(&xml);
  let accent = |base: char| {
    format!(
      "<XMApp><XMTok name=\"mycheck\" role=\"OVERACCENT\">\u{FFFD}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">{base}</XMTok></XMApp>"
    )
  };
  assert!(xml.contains(&accent('x')), "{xml}");
  assert!(
    xml.contains("<XMTok name=\"sumx\" role=\"BIGOP\">\u{FFFD}</XMTok>"),
    "{xml}"
  );
  // The align cell, whole: its accent over `u`.
  assert!(
    xml.contains(&format!(
      "<td align=\"right\"><Math mode=\"inline\" tex=\"\\displaystyle\\mycheck{{u}}\" \
       text=\"mycheck@(u)\"><XMath>{}</XMath></Math></td>",
      accent('u')
    )),
    "{xml}"
  );
}

/// fonts-nfss/undecodable_math_symbol_{warns_where_printed, first_digested_in_a_discarded_hold,
/// never_printed_says_nothing}: the Warning is raised where the document prints (absorbs) a box holding an
/// undecodable symbol, once per symbol font, never where one is only digested (OXIDIZED_DESIGN_DIVERGENCES #27). Printed
/// in the title, a `\mathchoice`, a tabular cell and through a savebox's `\usebox`: one Warning per font, naming the
/// symbol printed first (the title's `\neq`), the `\settowidth`'s `\doteq` never named. First digested under
/// `\refstepcounter`'s discarded diagnostics hold (a counter's `\the…`, review 89b r1 finding 1): 89b r1 printed U+FFFD
/// with 0 Warnings; now the print warns. Saved and never used (program.sty:834-838's `\powersetsymbol` box, sn-jnl.cls
/// copies) and only measured (`\settowidth`): 0 Warnings, no U+FFFD (r1: 2 Warnings). Each eager in process and
/// `--streaming` from the binary, alike.
#[test]
fn undecodable_math_symbol_warns_where_printed() {
  let neq = "Warning:missing_font_encoding:U/jkpsyc Symbol font symbolsC (U/jkpsyc) has no font map: the math symbol \
             \\neq (slot \"2C) prints U+FFFD, as will any other symbol of this font it cannot decode";
  let upomega = "Warning:missing_font_encoding:U/fsy Symbol font ugrfr (U/fsy) has no font map: the math symbol \
                 \\Upomega (slot \"0A) prints U+FFFD, as will any other symbol of this font it cannot decode";
  // The hold's case prints its U+FFFD where `a\neq b` is.
  let printed_neq = "<XMApp><XMTok name=\"neq\" role=\"RELOP\">\u{FFFD}</XMTok><XMTok font=\"italic\" \
                     role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMApp>";
  for (tex, expected, fffd, element) in [
    (
      repro!("undecodable_math_symbol_warns_where_printed.tex"),
      vec![neq, upomega],
      5,
      None,
    ),
    (
      repro!("undecodable_math_symbol_first_digested_in_a_discarded_hold.tex"),
      vec![neq],
      1,
      Some(printed_neq),
    ),
    (
      repro!("undecodable_math_symbol_never_printed_says_nothing.tex"),
      vec![],
      0,
      None,
    ),
  ] {
    let (log, xml) = convert_with(tex, Some(PRELOAD));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, expected.len()),
      "{log}"
    );
    assert_eq!(warnings(&log), expected, "{log}");
    assert_eq!(xml.matches('\u{FFFD}').count(), fffd, "{xml}");
    assert!(
      element.is_none_or(|element| squash(&xml).contains(element)),
      "{xml}"
    );
    let (log, streamed) = cli_conversion(tex, true);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, expected.len()),
      "streaming: {log}"
    );
    assert_eq!(warnings(&log), expected, "streaming: {log}");
    assert_eq!(
      streamed.matches('\u{FFFD}').count(),
      fffd,
      "streaming: {streamed}"
    );
    assert!(
      element.is_none_or(|element| squash(&streamed).contains(element)),
      "streaming: {streamed}"
    );
  }
}
