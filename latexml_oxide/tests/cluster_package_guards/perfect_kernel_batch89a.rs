//! Red/green guards for batch 89a: math glyphs lost from raw-loaded math-font packages (OXIDIZED_DESIGN_DIVERGENCES
//! #910, KNOWN_PERL_ERRORS #1201, #1202).
//!
//! - The symbol-font argument is read expanded, as latex.ltx resolves it through `\csname sym#3\endcsname`
//!   (`\DeclareMathSymbol`, `\DeclareMathAccent`, `\DeclareSymbolFont`'s name, family, series and shape): mathdesign's
//!   `{\MD@greek@family upright}` (1808.08390, 1910.04228, 1108.1913).
//! - A symbol whose declared font decodes nothing, declared at its reference slot (fontmath.ltx, amsfonts/amssymb),
//!   is the token LaTeXML's own kernel/amssymb binding makes of the name: kpfonts' `\Delta` in `U/jkpmia`,
//!   `\leqslant` and `\Cap` in an `AMSa` redeclared as `U/jkpsya` (1604.04618, 2008.13643), raw newtxsf's `\lfloor`,
//!   `\mid` and `\lgroup` in `LMS/ntxsy`/`LMX/ntxexx` (2506.22450), eulerpx's `\jmath` in `U/zeur` (2501.18458).
//! - Another slot stays undecoded: a variant swap (`\epsilon` at "22) has another glyph there.
//! - The msam, msbm, cmsy and cmmi slots Perl's fontmaps get wrong decode to the glyph the font draws (oz.sty's and
//!   zed-csp's `U/msa` `\Cap`), and the font metrics are keyed by it.
//!
//! One guard per path: the expanded arguments (each command's argument expanded; the family through `U_msa`), the
//! reference slot (a kernel name in a `U` font, a kernel delimiter in an `LMS` font, AMSa/AMSb names in redeclared
//! fonts, amssymb's binding beside them, a name with no reference left as it was), the binding's token where it
//! differs from the reference font's fontmap entry (glyph, role, meaning, stretchiness), the corrected fontmap slots,
//! their metrics (keyed by the corrected glyphs: the widths), and the negative (a kernel name at another slot). The first two also run through the binary, eager and
//! `--streaming`. Each repro is under `tools/perfect_kernel/repros/fonts-nfss/`, pdflatex-checked.
use std::{path::Path, process::Command};

use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  perfect_kernel_batch84b::text,
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

/// Every `<XMTok>` with a `name`, whole, in document order.
fn named_tokens(xml: &str) -> Vec<String> {
  let mut tokens = Vec::new();
  let mut rest = xml;
  while let Some(start) = rest.find("<XMTok ") {
    let tail = &rest[start..];
    let open_end = tail.find('>').map_or(tail.len(), |e| e + 1);
    let end = if tail[..open_end].ends_with("/>") {
      open_end
    } else {
      tail
        .find("</XMTok>")
        .map_or(tail.len(), |e| e + "</XMTok>".len())
    };
    if tail[..open_end].contains(" name=\"") {
      tokens.push(tail[..end].to_string());
    }
    rest = &tail[end..];
  }
  tokens
}

/// Converts `tex` in process, checks it converts with no Error or Warning, and returns its named tokens and the XML.
#[track_caller]
fn eager_tokens(tex: &str) -> (Vec<String>, String) {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  (named_tokens(&xml), xml)
}

/// Converts `tex` with the binary, `--streaming=true` when `streaming`, checks it converts with no Error or Warning,
/// and returns its named tokens.
#[track_caller]
fn cli_tokens(tex: &str, streaming: bool) -> Vec<String> {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
      if streaming {
        "--streaming=true"
      } else {
        "--streaming=false"
      },
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  named_tokens(&std::fs::read_to_string(workdir.path().join("t.xml")).expect("t.xml"))
}

/// Asserts `tex`'s named tokens are `expected`, in process and from the binary eager and `--streaming`.
#[track_caller]
fn assert_tokens_on_both_builds(tex: &str, expected: &[&str]) {
  let expected: Vec<String> = expected.iter().map(ToString::to_string).collect();
  let (tokens, xml) = eager_tokens(tex);
  assert_eq!(tokens, expected, "{xml}");
  assert_eq!(cli_tokens(tex, false), expected, "the binary, eager");
  assert_eq!(cli_tokens(tex, true), expected, "the binary, --streaming");
}

/// fonts-nfss/symbol_font_name_expands: `\DeclareMathSymbol`'s font `{\grfam}` and `{\grfam upright}` (mathdesign's
/// `{\MD@greek@family upright}`), `\DeclareSymbolFont`'s family `{\amsfam}` (decoded through `U_msa`) and name
/// `{\amsfam names}`, `\DeclareMathAccent`'s font `{\opfam}` — each read expanded — and a name broken over a line,
/// `{latex-font⏎msa}`, used as `{latex-font msa}` (1501.05180). Was U+FFFD, a newline, `6` and an empty accent.
#[test]
fn symbol_font_name_expands() {
  assert_tokens_on_both_builds(repro!("symbol_font_name_expands.tex"), &[
    r#"<XMTok font="italic" name="myalpha" role="ID">α</XMTok>"#,
    r#"<XMTok name="myDelta" role="ID">Δ</XMTok>"#,
    r#"<XMTok name="myOmega" role="ID">Ω</XMTok>"#,
    r#"<XMTok name="myleq" role="RELOP">⩽</XMTok>"#,
    r#"<XMTok name="mylesq" role="RELOP">⩽</XMTok>"#,
    r#"<XMTok name="myhat" role="OVERACCENT">ˆ</XMTok>"#,
    r#"<XMTok name="myonto" role="RELOP">↠</XMTok>"#,
  ]);
}

/// fonts-nfss/u_font_kernel_slot_and_ams_names: kpfonts' `\Delta`/`\Omega` in `U/jkpmia` at fontmath.ltx's
/// `operators` slots, raw newtxsf's `\lfloor`/`\rfloor` in `LMS/ntxsy` at the `symbols` slots, `\leqslant` and `\Bbbk`
/// in `AMSa`/`AMSb` redeclared as `U/jkpsya`/`U/jkpsyb` at amssymb's slots: the binding's tokens (`\leqslant`'s
/// meaning and the floors' `stretchy="false"` too, since 89a round 2). amssymb's own `\square` (its binding) is as
/// before; kpfonts' `\neq` in `U/jkpsyc` at 44 has no reference slot there and keeps the raw slot
/// (OXIDIZED_DESIGN_DIVERGENCES #27). Was U+FFFD, a newline, `6`, `b`, `c` and `|`.
#[test]
fn kernel_and_ams_names_at_their_reference_slots() {
  assert_tokens_on_both_builds(repro!("u_font_kernel_slot_and_ams_names.tex"), &[
    r#"<XMTok name="Delta" role="ID">Δ</XMTok>"#,
    r#"<XMTok name="Omega" role="ID">Ω</XMTok>"#,
    r#"<XMTok meaning="less-than-or-equals" name="leqslant" role="RELOP">⩽</XMTok>"#,
    r#"<XMTok name="neq" role="RELOP">,</XMTok>"#,
    r#"<XMTok name="lfloor" role="OPEN" stretchy="false">⌊</XMTok>"#,
    r#"<XMTok name="rfloor" role="CLOSE" stretchy="false">⌋</XMTok>"#,
    r#"<XMTok name="square" role="UNKNOWN">□</XMTok>"#,
    r#"<XMTok name="Bbbk" role="ID">𝕜</XMTok>"#,
  ]);
}

/// fonts-nfss/font_package_symbols_are_the_bindings: kpfonts' `\Cup`/`\Cap`/`\backsimeq` in `U/jkpsya` and
/// `\ntriangleleft`/`\ntriangleright` in `U/jkpsyb`, eulerpx's `\jmath` in `U/zeur`, newtxsf's `\mid` in `LMS/ntxsy`
/// and `\lgroup`/`\rgroup` delimiters in `LMX/ntxexx`, each at its amssymb/fontmath.ltx slot (decimal, as the
/// packages write it): the token amssymb's and the kernel's bindings make — the glyph pdflatex prints, the binding's
/// role, meaning and stretchiness — where the reference fonts' fontmap entries differ. 89a round 1 printed ⋓ for
/// `\Cap`, ⋒ for `\Cup`, the two triangles swapped, ≌, `j`, `|` and the extensible pieces ⎩/⎭, with the class's role.
/// The control-symbol delimiter `\|` in `LMS/ntxsy` at "6B (fontmath.ltx:467) is the kernel's ‖, as in a document
/// without the redeclaration (the parser makes the pair OPEN/CLOSE); 89a round 2's table had no control symbols and
/// printed `k`.
#[test]
fn font_packages_get_the_bindings_symbols() {
  let (tokens, xml) = eager_tokens(repro!("font_package_symbols_are_the_bindings.tex"));
  assert_eq!(
    tokens,
    [
      r#"<XMTok meaning="double-union" name="Cup" role="ADDOP">⋓</XMTok>"#,
      r#"<XMTok meaning="double-intersection" name="Cap" role="ADDOP">⋒</XMTok>"#,
      r#"<XMTok meaning="not-subgroup-of" name="ntriangleleft" role="RELOP">⋪</XMTok>"#,
      r#"<XMTok meaning="not-contains" name="ntriangleright" role="RELOP">⋫</XMTok>"#,
      r#"<XMTok name="backsimeq" role="RELOP">⋍</XMTok>"#,
      r#"<XMTok font="italic" name="jmath" role="ID">ȷ</XMTok>"#,
      r#"<XMTok name="mid" role="MIDDLE">∣</XMTok>"#,
      r#"<XMTok name="lgroup" role="OPEN" stretchy="false">⟮</XMTok>"#,
      r#"<XMTok name="rgroup" role="CLOSE" stretchy="false">⟯</XMTok>"#,
      r#"<XMTok name="||" role="OPEN">‖</XMTok>"#,
      r#"<XMTok name="||" role="CLOSE">‖</XMTok>"#,
    ],
    "{xml}"
  );
}

/// fonts-nfss/ams_font_slots_decode_as_msam_msbm_draw: oz.sty's and zed-csp's shape, a symbol font declared `U/msa`
/// (and `U/msb`), decoded through the msam/msbm maps, and the kernel's `symbols`/`letters` through OMS/OML, at the
/// slots Perl's fontmaps get wrong (KNOWN_PERL_ERRORS #1202): ⋒ at msam "65, ⋓ at "64, ⋍ at "77, ⋪/⋫ at msbm
/// "36/"37, ⊎ at cmsy "5D, ȷ at cmmi "7C. No reference name, so the class gives the role. Was the Perl glyphs.
#[test]
fn ams_font_slots_decode_as_the_fonts_draw() {
  let (tokens, xml) = eager_tokens(repro!("ams_font_slots_decode_as_msam_msbm_draw.tex"));
  assert_eq!(
    tokens,
    [
      r#"<XMTok name="oCup" role="BINOP">⋓</XMTok>"#,
      r#"<XMTok name="oCap" role="BINOP">⋒</XMTok>"#,
      r#"<XMTok name="obacksimeq" role="RELOP">⋍</XMTok>"#,
      r#"<XMTok name="ontl" role="RELOP">⋪</XMTok>"#,
      r#"<XMTok name="ontr" role="RELOP">⋫</XMTok>"#,
      r#"<XMTok name="ouplus" role="BINOP">⊎</XMTok>"#,
      r#"<XMTok font="italic" name="ojmath" role="ID">ȷ</XMTok>"#,
    ],
    "{xml}"
  );
}

/// fonts-nfss/kernel_name_at_another_slot_stays_undecoded: `\epsilon` at "22 and `\varphi` at "1E of `U/jkpmia` —
/// slots fontmath.ltx gives `\varepsilon` and `\phi` — are not decoded through OML; the raw
/// slot stays (`"`, and U+FFFD for a control slot; OXIDIZED_DESIGN_DIVERGENCES #27). The same as the base.
#[test]
fn kernel_name_at_another_slot_stays_undecoded() {
  let (tokens, xml) = eager_tokens(repro!("kernel_name_at_another_slot_stays_undecoded.tex"));
  assert_eq!(
    tokens,
    [
      r#"<XMTok name="epsilon" role="ID" rpadding="5.0pt">"</XMTok>"#,
      "<XMTok name=\"varphi\" role=\"ID\">\u{FFFD}</XMTok>",
    ],
    "{xml}"
  );
}

/// fonts-nfss/corrected_slot_glyphs_have_their_tfm_widths: `\settowidth` of `$\uplus$`, `$\backsimeq$` and `$\jmath$`
/// gives pdflatex's 6.66669pt, 7.7778pt and 3.8403pt (cmsy10 "5D, msam10 "77, cmmi10 "7C): the metrics are keyed by
/// the glyph the font draws, ⊎ ⋍ ȷ (KNOWN_PERL_ERRORS #1202). 89a round 2 and Perl gave 7.5pt, 7.5pt (the 0.75em
/// default: the metrics sat under ⊌ and ≌) and 3.05557pt (cmr10's dotless j: cmmi had no ȷ).
#[test]
fn corrected_slot_glyphs_have_their_tfm_widths() {
  let (log, xml) = convert_with(
    repro!("corrected_slot_glyphs_have_their_tfm_widths.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let para = xml
    .split("<p ")
    .nth(1)
    .and_then(|p| p.split("</p>").next())
    .expect("a paragraph");
  assert_eq!(
    text(&format!("<p {para}")),
    "U6.66669pt; B7.7778pt; J3.8403pt;",
    "{xml}"
  );
}
