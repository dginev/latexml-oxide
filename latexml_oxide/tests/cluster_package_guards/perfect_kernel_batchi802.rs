//! Guard for issue 802: an XMDual collapsed to a branch that has its own id renames every reference to the dual to
//! the branch's id (Perl Document.pm:1643), through `Document::idref_index` rather than a whole-document
//! `//*[@idref='…']` per dual, which libxml2 refuses past 10,000,000 nodes. The repro's pdflatex text is in its header.
use latexml::util::test::{convert_with, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One row per math of the repro (`math-parse/xmdual_collapse_renames_refs.tex`, 0 errors, 0 warnings), its whole
/// XMath: the reference in the middle dual's removed content branch is renamed too, then removed with it (row 1);
/// every one of several references is renamed (row 2).
#[test]
fn collapsed_dual_references_take_the_branch_id() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/xmdual_collapse_renames_refs.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let rows: [(&str, &str); 2] = [
    (
      r#"xml:id="p1.m1.2a""#,
      r#"<XMath xml:id="p1.m1.2a"><XMDual xml:id="p1.m1.2a.1"><XMTok font="italic" role="UNKNOWN">y</XMTok><XMApp xml:id="p1.m1.2a.1.2"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.2">x</XMTok><XMRef idref="p1.m1.2" xml:id="p1.m1.2a.1.2.2"/></XMApp></XMDual></XMath>"#,
    ),
    (
      r#"xml:id="p1.m2.2a""#,
      r#"<XMath xml:id="p1.m2.2a"><XMDual xml:id="p1.m2.2a.1"><XMTok font="italic" role="UNKNOWN">y</XMTok><XMApp xml:id="p1.m2.2a.1.2"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.2">x</XMTok><XMRef idref="p1.m2.2" xml:id="p1.m2.2a.1.2.2"/><XMRef idref="p1.m2.2" xml:id="p1.m2.2a.1.2.3"/></XMApp></XMDual></XMath>"#,
    ),
  ];
  for (id, expected) in rows {
    assert_eq!(
      xml_element(&xml, "XMath", &[id]).as_deref(),
      Some(expected),
      "{xml}"
    );
  }
}
