//! Red/green guards for batch 80a (the hyphenation languages the format numbers keep their numbers) and 80b (a
//! listings style that is a control sequence runs, with the token in `\lst@token`) and 80c (import.sty's path model). Each guard
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

/// 80b: a listings style that is a control sequence runs as markup whatever its name ends in, and the token being
/// output is in `\lst@token` while it runs (listings.sty:481, :579-585). Red (int94a): `identifierstyle=\idstyle` was
/// taken for a class `\ids` and never ran, the default style's `}}` stayed after every identifier ("Attempt to close
/// boxing group"), and `\lst@token` was empty (KPE #1169, #1170). Witness 1907.13012. Repro
/// listings-algorithms/listings_identifier_style_reads_the_token.
#[test]
fn a_listings_identifier_style_reads_the_token() {
  let (log, xml) = convert_files_with(
    repro!("listings-algorithms/listings_identifier_style_reads_the_token.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (color, word) in [
    ("#0000FF", "Query"),
    ("#FF0000", "field"),
    ("#FF0000", "value"),
  ] {
    let element = format!(r#"color="{color}""#);
    let at = xml
      .find(&format!(">{word}</text>"))
      .unwrap_or_else(|| panic!("{word}: {xml}"));
    let open = xml[..at].rfind("<text ").unwrap();
    assert!(xml[open..at].contains(&element), "{word} {color}: {xml}");
  }
  assert!(xml.contains("After."), "{xml}");
}

/// 80c: import.sty's own path model (import_sty.rs) — `\import` resets `\import@path`, `\subimport` extends it, the
/// file read is `\import@path <file>` with the enclosing imports' directories still searched — and a directory is no
/// file (pathname.rs, kpathsea's `READABLE`). Red (int94b): the nested `\import{intro}{intro}` read the directory
/// importnest/intro ("Is a directory"), the section, its `\subimport{}{parts/part.tex}` part and the label lost
/// (KPE #1171). Witness 2408.09583. Repro loader/import_nested_absolute_import.
#[test]
fn a_nested_import_reads_its_files_as_import_sty_finds_them() {
  let (log, xml) = convert_files_with(
    repro!("loader/import_nested_absolute_import.tex"),
    &[
      (
        "importnest/chapters.tex",
        include_str!("../../../tools/perfect_kernel/repros/loader/importnest/chapters.tex"),
      ),
      (
        "importnest/intro/intro.tex",
        include_str!("../../../tools/perfect_kernel/repros/loader/importnest/intro/intro.tex"),
      ),
      (
        "importnest/intro/parts/part.tex",
        include_str!("../../../tools/perfect_kernel/repros/loader/importnest/intro/parts/part.tex"),
      ),
    ],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for text in [
    "The introduction text.",
    "A part of the introduction.",
    "Back in the chapters file.",
  ] {
    assert!(xml.contains(text), "{text}: {xml}");
  }
  assert!(xml.contains(r#"labels="LABEL:sec:intro""#), "{xml}");
  assert!(
    xml.contains(r#"<ref labelref="LABEL:sec:intro"/>"#),
    "{xml}"
  );
}
