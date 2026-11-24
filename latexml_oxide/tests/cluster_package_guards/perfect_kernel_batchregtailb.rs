//! Red/green guards for batch REGTAILB (row R REGTAIL: papers an old build converted cleanly that later erred or
//! went Fatal): a braced file name, amsgen's `\RIfM@`, breqn's load-time script catcodes, tipa's accented letter in a
//! formula, a list label that is the author's own `\fnum@<counter>`.
use latexml::util::test::{assert_element, convert_with};

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
