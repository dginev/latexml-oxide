//! Red/green guard for batch 66v: the floatrow binding records float.sty and rotfloat.sty loaded, as floatrow.sty:34-37
//! marks them (66m E1, user ruling 2026-10-14).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// `\usepackage{float}` after floatrow loads nothing: floatrow's `\float@end` closes the document's figure, which keeps
/// its caption (2010.00977, 2003.11778, 2011.10793, 2105.04869).
#[test]
fn floatrow_marks_float_loaded() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/loader/floatrow_marks_float_loaded.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(log.contains("(Loading floatrow_sty.rs"), "{log}");
  assert!(!log.contains("(Loading float_sty.rs"), "{log}");
  assert_eq!(xml.matches("<caption>").count(), 1, "{xml}");
  let float = xml
    .split(r#"<float class="ltx_float_figure""#)
    .nth(1)
    .and_then(|rest| rest.split("</float>").next())
    .unwrap_or_else(|| panic!("no figure float: {xml}"));
  assert!(float.contains("A caption."), "{xml}");
}
