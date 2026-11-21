//! Red/green guards for batch REGFIXW: a Latin-1 character captured before `\usepackage[utf8]{inputenc}` keeps its
//! meaning. Repro `tools/perfect_kernel/repros/unicode-catcodes/`; pdflatex-checked.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The kernel's `\DeclareUnicodeCharacter` makes `§` active in the format; a `\def` or a `\crefformat` (arxiv.sty:55-60)
/// stores it before `[utf8]{inputenc}`. utf8.def:174-193 redefines every byte 0x80-0xFF, so in TeX no byte keeps
/// `\@inpenc@undefined` (inputenc.sty:84): pdflatex prints "B[§]" and "See §1.", 0 errors. Red (INT6D): 2
/// Error:latex:(inputenc) "Keyboard character used is undefined", B[] (Perl: 1 Error, B[]). Witness 2605.20531.
#[test]
fn latin1_char_before_utf8_inputenc_keeps_meaning() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/unicode-catcodes/latin1_char_before_utf8_inputenc_keeps_meaning.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S1.p1.1\""],
    "<p xml:id=\"S1.p1.1\">A[13] B[§] C[café]</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S1.p2.1\""],
    "<p xml:id=\"S1.p2.1\">See <ref labelref=\"LABEL:sec:one\" show=\"crefformatcap|creftypecap~creflabel\"/>.</p>",
  );
}
