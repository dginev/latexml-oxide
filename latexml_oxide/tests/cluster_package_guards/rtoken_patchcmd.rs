//! etoolbox's `\patchcmd` rebuilds the macro from its `\meaning` and keeps its
//! `\protected\long\outer` prefix, or takes the one its `[prefix]` option gives
//! (etoolbox.sty:1350-1371); the binding installed a plain macro (witness
//! yquant/yquant-doc, 1001 errors and a Fatal on 56ji-rel). The name of `\def`,
//! `\let`, `\chardef` … must be a control sequence or an active character:
//! anything else is TeX's "Missing control sequence inserted", the token is
//! read again and the frozen `\inaccessible` is defined instead (tex.web §1215
//! `get_r_token`); a `\gdef{` had made every later `{` a macro. The NFSS font
//! switches are robust, as latex.ltx declares them: as plain macros they
//! expanded inside `\protected@edef` into a definition of the letter c. Text
//! and error counts are pdflatex's; repros in
//! `tools/perfect_kernel/repros/macro-state/`, `…/expansion-primitives/`,
//! `…/fonts-nfss/` and `…/index/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// The `<para xml:id="p{n}">` holding one `<p>` with `content`.
fn assert_para(xml: &str, n: usize, content: &str) {
  assert_element(
    xml,
    "para",
    &[&format!(r#"xml:id="p{n}""#)],
    &format!(r#"<para xml:id="p{n}"><p>{content}</p></para>"#),
  );
}

/// Convert a clean repro: pdflatex has no error and no warning on it.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert(tex, true);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// A `\patchcmd`ed copy of a `\protected` macro stays protected, so an `\edef`
/// leaves it unexpanded (yquant-config.tex:594-596 patches a copy of
/// yquant-shapes.tex:26's `\pgfshapeclippath`, which yquant-draw.tex:201-211
/// puts in an `\edef`). pdflatex "PROTECTED-KEPTP"; Rust expanded it, 1 error,
/// "PROTECTED-LOSTNP".
#[test]
fn patchcmd_keeps_the_protected_prefix() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/patchcmd_keeps_protected_yquant.tex"
  ));
  assert_para(&xml, 1, "PROTECTED-KEPTP");
}

/// Without the option the patched macro keeps the original's prefix, `[]`
/// strips it and `[\long]` replaces it; the search text is replaced once. Rust
/// gave "macro:#1->y#1y" for all three.
#[test]
fn patchcmd_prefix_option_and_single_replacement() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/patchcmd_prefix_option.tex"
  ));
  let line = |text: &str| format!(r#"<text font="typewriter">{text}</text>"#);
  assert_para(&xml, 1, &line(r"A [\protected\long macro:#1-&gt;y#1x]"));
  assert_para(&xml, 2, &line("B [macro:#1-&gt;y#1x]"));
  assert_para(&xml, 3, &line(r"C [\long macro:#1-&gt;y#1x]"));
}

/// `\gdef{}X{Y}`: the `{` is no name, so TeX says "Missing control sequence
/// inserted", defines `\inaccessible` as `\gdef\inaccessible{}` and typesets
/// `X{Y}` (tex.web §1215). yquant-doc.tex:3832's `\gdef{…` had installed a
/// macro on the Begin catcode: Rust 4 errors, X and Y lost, the bold leaked.
#[test]
fn def_of_a_non_cs_defines_inaccessible() {
  let (log, xml) = convert(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/def_noncs_target_yquant.tex"
    ),
    true,
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    log.matches("Missing control sequence inserted").count(),
    1,
    "{log}"
  );
  assert_para(
    &xml,
    1,
    r#"XY Before a and <text font="bold">b</text> after."#,
  );
}

/// `\let`, `\chardef` and `\futurelet` name through the same §1215 read: the
/// letter, the digit and the letter are read again after `\inaccessible` (one
/// error each). Rust defined them: 0 errors, "A  [the letter a]", "B [1]",
/// "C  yz [x]".
#[test]
fn let_family_of_a_non_cs_reads_the_token_again() {
  let (log, xml) = convert(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/let_family_noncs_target.tex"
    ),
    true,
  );
  assert_eq!(error_count(&log), 3, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    log.matches("Missing control sequence inserted").count(),
    3,
    "{log}"
  );
  assert_para(&xml, 1, "A =b [the letter a]");
  assert_para(&xml, 2, "B =\u{2018}C [1]");
  assert_para(&xml, 3, "C x yz [x]");
}

/// latex.ltx makes the NFSS font switches robust, so `\protected@edef` keeps
/// `\ttfamily` and the font reaches the text (pdflatex "A B C D", B and D in
/// typewriter). As plain macros `\ttfamily` expanded there to `\edef
/// cmr{cmtt}`: 56ji-rel defined the letter c and set B upright; under tex.web
/// §1215's name check (W17 R2) it is 1 error (pythonimmediate's
/// `\DescribeOption`, 12).
#[test]
fn font_switches_are_robust_in_protected_edef() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/protected_edef_ttfamily_pythonimmediate.tex"
  ));
  assert_para(
    &xml,
    1,
    r#"A <text font="typewriter">B</text> C <text font="typewriter">D</text>"#,
  );
}

/// An `\index` entry is expanded as `\protected@write` would, and a robust
/// `\ttfamily` stays a font switch there (guitar's `\DescribeEnv` →
/// doc.sty's `\SpecialEnvIndex`). Perl's phrase; 56ji-rel dropped the font, and
/// under §1215's name check (W17 R2) it is an error per entry.
#[test]
fn index_entry_keeps_ttfamily() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/index/index_ttfamily_guitar.tex"
  ));
  assert_element(
    &xml,
    "indexphrase",
    &[],
    r#"<indexphrase key="guitar"><text font="typewriter">guitar</text></indexphrase>"#,
  );
}

/// A `\noexpand`ed name is the control sequence it marks (tex.web §358 sets
/// `cur_cs`): `\expandafter\def\noexpand\foo` defines `\foo` (pdflatex "A [X]
/// [BAR] []"; Rust defined the `\special_relax` marker, 2 errors).
#[test]
fn noexpand_marked_name_is_its_control_sequence() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/noexpand_definition_name.tex"
  ));
  assert_para(&xml, 1, "A [X] [BAR] []");
}

/// tcilatex's `\activesoff`, which `\FRAME` runs under babel, redefines the
/// ACTIVE `"` `;` `:` `'` `~`: tcilatex.tex.ltxml:369-389 defines it inside
/// `\bgroup\makeactives…\egroup`. Defined at normal catcodes it named four
/// characters that are no control sequences (tex.web §1215): 4 errors per
/// frame. The figure, with its `\Qcb` caption, is Perl's.
#[test]
fn tcilatex_frame_under_babel_defines_active_punctuation() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/tcilatex_frame_babel_activesoff.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    concat!(
      r#"<figure inlist="lof" labels="LABEL:fig1" placement="tbp" xml:id="S0.F1">"#,
      r#"<tags><tag>Figure 1</tag><tag role="refnum">1</tag>"#,
      r#"<tag role="typerefnum">Figure 1</tag></tags>"#,
      r#"<p align="center"><text width="144.5pt" yoffset="72.3pt"/></p>"#,
      r#"<toccaption><tag close=" ">1</tag>A caption</toccaption>"#,
      r#"<caption><tag close=": ">Figure 1</tag>A caption</caption></figure>"#
    ),
  );
  assert_para(&xml, 1, "Before.");
  assert_para(&xml, 2, "After.");
}

/// `\ifx` of a `\let` copy of a robust font switch and the switch is true, as
/// in pdflatex (tex.web §507: TeX's copy is the same wrapper). Rust's `\let`
/// copies the body under its own wrapper (OXIDIZED_DESIGN_DIVERGENCES #315),
/// so `\ifx` compares the bodies, which must be one definition: two robust
/// commands declared apart with the same body stay unequal ("A: TTTFF."; W17
/// before: "FFFFF", then "TTTFT"). amsthm's
/// `\nonslanted` (amsthm.sty:209-212) keeps small caps ("f[sc]"; the binding's
/// `\let\nonslanted\upshape`, Perl's, gave "f[n]"); in the body it compares
/// the `\protected` shape switches, a plain meaning comparison.
#[test]
fn ifx_sees_let_copies_of_robust_font_switches() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/ifx_let_copy_of_robust_switch_nonslanted.tex"
  ));
  assert_para(&xml, 1, "A: TTTFF.");
  assert_para(
    &xml,
    2,
    concat!(
      r#"B: <text font="italic">a[it]</text>b[n] <text font="slanted">c[sl]</text>d[n] "#,
      r#"<text font="smallcaps">e[sc]f[sc]</text>"#
    ),
  );
}

/// In the body the shape switches are `\protected` macros: `\begin{document}`
/// runs latex.ltx's `\reinstall@nfss@defs` (:12489-12514), so a plain `\edef`
/// keeps `\itshape` (pdflatex "macro:->\itshape "). With only the preamble's
/// robust wrappers the `\edef` stored `\edef n{it}` (2 errors, "it" upright).
/// `\normalshape` is `\protected` everywhere (latex.ltx:12486-12488): as a
/// `\let` of the robust `\upshape` it was 1 error and "ab" italic.
#[test]
fn body_shape_switches_are_protected() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/body_shape_switches_protected.tex"
  ));
  assert_para(
    &xml,
    1,
    r#"A: <text font="typewriter">macro:-&gt;\itshape </text>."#,
  );
  assert_para(
    &xml,
    2,
    r#"B: <text font="typewriter">macro:-&gt;\scshape </text>."#,
  );
  assert_para(
    &xml,
    3,
    r#"C: <text font="italic">it</text> <text font="smallcaps">sc</text> n."#,
  );
  assert_para(
    &xml,
    4,
    r#"D: <text font="typewriter">macro:-&gt;\normalshape </text>; <text font="italic">a</text>b c."#,
  );
}
