//! Red/green guards for batch 66o: the `\endgroup` cluster's natbib root (root-causer report
//! `agent_reports/2026-10-14_endgroup_cluster.md`, E3).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// A `\bibitem` label piece with an unbalanced `$` is closed as a formula of its own: the math no longer runs past the
/// `\bibitem` into the refnum/key tags and the rest of the bibliography (1208.0605); a group's stray last `$` is dropped
/// (revtex's `\citenamefont {Sasaki$}`: 1205.3745, 1508.02141).
#[test]
fn natbib_unbalanced_dollar_in_label() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/natbib_unbalanced_dollar_in_label.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      r#"<tag role="fullauthors">Mantz, <Math mode="inline" tex="\,{Ebeling},\&amp;{Rapetti}""#
    ),
    "{xml}"
  );
  assert!(
    xml.contains("<tag role=\"refnum\">Mantz et\u{a0}al. (2008)</tag>"),
    "{xml}"
  );
  assert!(xml.contains(r#"<tag role="key">mae08</tag>"#), "{xml}");
  assert!(xml.contains(r#"<tag role="authors">Sasaki</tag>"#), "{xml}");
  assert!(
    xml.contains(r#"<tag role="refnum">Sasaki (2011)</tag>"#),
    "{xml}"
  );
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}
