//! Red/green guards for batch REGTAILB (row R REGTAIL: papers an old build converted cleanly that later erred or
//! went Fatal): a braced file name, amsgen's `\RIfM@`, breqn's load-time script catcodes, tipa's accented letter in a
//! formula, a list label that is the author's own `\fnum@<counter>`.
use latexml::util::test::{assert_element, convert_with, kpse_has};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// loader/braced_file_name_group: a file name that is one braced group loses one brace level, as latex.ltx's
/// `\@firstofone#1` reads it (`\set@curr@file`, `\@kernel@make@file@csname`), through every file-name reader:
/// `\include` (with `\includeonly`), `\input`, `\IfFileExists`, `\InputIfFileExists` (2406.05605's dukedissertation
/// `\include{{./Chapter1/introduction}}`, 2408.05715, 2309.07288). Red (INT081G): the included file silently dropped
/// ("No file {./tfbone}.tex.") and `Error:missing_file:{./tfbtwo}`.
#[test]
fn braced_file_name_group_is_read_without_its_braces() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/loader/braced_file_name_group_2406_05605.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("No file"), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">One.\nTwo.\n yes Three.</p>",
  );
}

/// ams-internals/amsgen_rifm: amsgen.sty:49's `\RIfM@` (Perl's binding spells it `\RifM@`, KNOWN_PERL_ERRORS #1324),
/// which a document's `\QED` box calls (1701.06055: 101 errors and a Fatal). Red (INT081G): 5 errors.
#[test]
fn amsgen_defines_rifm() {
  let (log, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/ams-internals/amsgen_rifm_1701_06055.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">t <Math mode=\"inline\" tex=\"m\" text=\"m\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.1\"><XMTok \
     font=\"italic\" role=\"UNKNOWN\">m</XMTok></XMath></Math></p>",
  );
}

/// loader/breqn_resets_script_catcodes_at_load: breqn.sty:41 sets `^`/`_` to 7/8 at load, so gb4e's active
/// `\automath` scripts are off for tikz loaded after (2402.13372). Red (INT081G): "13/13", then tikz looped to
/// `Fatal:Timeout:PushbackLimit` with 227k Warnings. The one Warning is the binding's "breqn.sty is minimally stubbed".
/// Distinct from `perfect_kernel_batch63::breqn_text_underscore` (the body's catcode 12 from `\begin{document}`).
#[test]
fn breqn_resets_script_catcodes_at_load() {
  if let Some(missing) = ["gb4e.sty", "breqn.sty", "tikz.sty"]
    .into_iter()
    .find(|f| !kpse_has(f))
  {
    eprintln!("SKIP breqn_resets_script_catcodes_at_load: {missing} is not in this TeX Live");
    return;
  }
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/loader/breqn_resets_script_catcodes_at_load_2402_13372.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">8/7, 12 <Math mode=\"inline\" tex=\"x_{1}\" text=\"x _ 1\" xml:id=\"p1.m1\"><XMath \
     xml:id=\"p1.m1.1\"><XMApp xml:id=\"p1.m1.1.1\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/><XMTok \
     font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok fontsize=\"70%\" meaning=\"1\" \
     role=\"NUMBER\">1</XMTok></XMApp></XMath></Math></p>",
  );
}

/// fonts-nfss/tipa_accent_letter_in_math_is_a_box: t3enc.def's accent sets its letter in an `\hbox` (:270, :284), so
/// a formula in the letter is a formula of its own inside the enclosing one (1310.2513's `\sliding{$g_4$}` in
/// `$\gst(K)$`). Red (INT081G): "Script _ can only appear in math mode". The two Warnings: tipx's slot 9 has no
/// combining mark, the letter is printed without it; and U/tipx has no glyph map, its slots read as OT1 (DINGBAT,
/// OXIDIZED_DESIGN_DIVERGENCES #645; was an Info).
#[test]
fn tipa_accent_letter_in_math_is_a_box() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/tipa_accent_letter_in_math_is_a_box_1310_2513.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert!(
    log.contains("has no combining mark for the accent in slot 9"),
    "{log}"
  );
  assert!(
    log.contains("Warning:unexpected:tipx Font family \"tipx\" in encoding \"U\" has no glyph map"),
    "{log}"
  );
  assert_element(
    &xml,
    "Math",
    &[],
    "<Math mode=\"inline\" tex=\"\\hbox{$g_{4}$}(K)\" text=\"g _ 4 * K\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.2\"><XMApp \
     xml:id=\"p1.m1.2.1\"><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMApp xml:id=\"p1.m1.2.1.2\"><XMTok \
     role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">g</XMTok><XMTok fontsize=\"70%\" \
     meaning=\"4\" role=\"NUMBER\">4</XMTok></XMApp><XMDual xml:id=\"p1.m1.2.1.3\"><XMRef idref=\"p1.m1.1\" \
     xml:id=\"p1.m1.2.1.3.1\"/><XMWrap xml:id=\"p1.m1.2.1.3.2\"><XMTok role=\"OPEN\" stretchy=\"false\">(</XMTok><XMTok \
     font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.1\">K</XMTok><XMTok role=\"CLOSE\" \
     stretchy=\"false\">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>",
  );
}

/// lists/list_label_is_the_authors_fnum: a list labelled by the author's own `\fnum@figcaption` keeps that meaning
/// for the label (`counter::dialect::define_item_fnum`), both for the list's label and an `\item[\fnum@figcaption]`
/// (cond-mat9505034; KNOWN_PERL_ERRORS #1325). Red (INT081G): `Fatal:Timeout:Recursion`; Perl 1 Error. The explicit
/// item's "Fig. :" is Perl's `\let\the<counter>\@empty` for a tagged item (latex_constructs.pool.ltxml:1390).
#[test]
fn list_label_is_the_authors_fnum() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/lists/list_label_is_the_authors_fnum_cond_mat9505034.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "itemize",
    &[],
    "<itemize xml:id=\"p1.1\"><item xml:id=\"S0.I1.i1\"><tags><tag><text font=\"bold\" xml:id=\"S0.I1.i1.1\">Fig. \
     1:</text></tag><tag role=\"refnum\">1</tag></tags><para xml:id=\"S0.I1.i1.p1\"><p xml:id=\"S0.I1.i1.p1.1\">Total \
     magnetization.</p></para></item><item xml:id=\"S0.I1.ix1\"><tags><tag><text font=\"bold\" \
     xml:id=\"S0.I1.ix1.1\">Fig. :</text></tag><tag role=\"typerefnum\">item <text font=\"bold\" \
     xml:id=\"S0.I1.ix1.2\">Fig. :</text></tag></tags><para xml:id=\"S0.I1.ix1.p1\"><p \
     xml:id=\"S0.I1.ix1.p1.1\">Explicit.</p></para></item><item xml:id=\"S0.I1.i2\"><tags><tag><text font=\"bold\" \
     xml:id=\"S0.I1.i2.1\">Fig. 2:</text></tag><tag role=\"refnum\">2</tag></tags><para xml:id=\"S0.I1.i2.p1\"><p \
     xml:id=\"S0.I1.i2.p1.1\">Third.</p></para></item></itemize>",
  );
}
