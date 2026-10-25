//! Red/green guards for batch 80a: the hyphenation languages the format numbers keep their numbers. Each guard
//! converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// latex_constructs declares the `\l@<language>` names babel checks for only where the format left them undefined
/// (sect13.rs), so TeX Live's numbering stands: `\l@english` 0, the language TeX starts in, and `\l@nohyphenation`
/// 2. Red (int94a): "other: 90, 94." — every name numbered again past the format's table (KPE #1168). Witness
/// 0802.3014 (languages.sty's "Inconsistent language numbering"). Repro babel-lang/language_numbers_follow_the_format.
#[test]
fn the_format_language_numbers_stand() {
  let (log, xml) = convert_files_with(
    repro!("babel-lang/language_numbers_follow_the_format.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<p xml:id=\"p1.1\">English: 0,\n2.</p>"),
    "{xml}"
  );
}
