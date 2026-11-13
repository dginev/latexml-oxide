//! Red/green guards for batch 79z (complete breakage, continued: user 2026-10-24): a listings style applies as
//! listings.sty's `{\lst@thestyle{…}}`, and an escape is tried before the other delimiters. Each guard converts its
//! repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// listings_sty.rs `lst_add_delimiter`/`lst_set_class_style`: a markup style opens its class as `{<style>{` and closes
/// it with `}}`, listings.sty:579-585's `{\lst@thestyle{…}}`, so the btHL idiom's `\bgroup\aftergroup\bt@HL@endenv`
/// is closed after the delimited text and its highlight box ends there: the listing keeps its text, one box is drawn,
/// and the section after the listing and its reference are kept. Red (int93b): 10 errors, the box left open past the
/// listing. Witnesses 2006.16205, 2104.08012. Repro listings-algorithms/listings_bthl_delimiter_style_group.
#[test]
fn a_bthl_delimiter_style_closes_its_box() {
  let (log, xml) = convert_files_with(
    repro!("listings-algorithms/listings_bthl_delimiter_style_group.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<svg:svg ").count(), 1, "{xml}");
  assert!(xml.contains("highlighted"), "{xml}");
  assert!(xml.contains("return"), "{xml}");
  assert!(xml.contains(r#"labels="LABEL:sec:after""#), "{xml}");
  assert!(
    xml.contains(r#"<ref labelref="LABEL:sec:after"/>"#),
    "{xml}"
  );
}

/// listings_sty.rs: the escape delimiters come first in the delimiter alternation (lstmisc.sty:337-347 `\lst@DefEsc`
/// is tested before the language's delimiters sharing its first character), so `escapeinside={\#*}{*}` under Python's
/// `#` comments runs `#*\label{line:n}*` as TeX: no `\label` text in the listing, line 1 labelled, the reference
/// resolved. Red (int93b): the line printed as a comment with its `\label`, the reference dangling. Witness
/// 2104.08012. Repro listings-algorithms/listings_escapeinside_before_comment.
#[test]
fn an_escape_is_tried_before_a_comment_sharing_its_character() {
  let (log, xml) = convert_files_with(
    repro!("listings-algorithms/listings_escapeinside_before_comment.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains(r"\label"), "{xml}");
  assert!(xml.contains(r#"labels="LABEL:line:n""#), "{xml}");
  assert!(xml.contains(r#"<ref labelref="LABEL:line:n"/>"#), "{xml}");
  // The second line's `#` still opens a Python comment.
  assert!(
    xml.contains(r#"<text class="ltx_lst_comment" font="italic" xml:id="lstnumberx2.5">#"#),
    "{xml}"
  );
}
