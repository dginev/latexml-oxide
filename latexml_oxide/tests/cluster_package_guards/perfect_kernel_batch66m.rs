//! Red/green guards for batch 66m: the low-risk mechanisms of the arXiv `\endgroup` error cluster (root-causer
//! report `agent_reports/2026-10-14_endgroup_cluster.md`, E1 E4 E5 E6 E8 E9). E1's loader half — another file's
//! `\ver@` mark — awaits a ruling (RED repro `loader/ver_mark_skips_the_marked_package`, KNOWN_PERL_ERRORS #668).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// E4: `\lstinline` is robust, as listings.sty:1941's `\newcommand\lstinline[1][]` is: ifthen's `\equal` reads it
/// unexpanded in its `\xdef` (ifthen.sty:101-110), so the test is false and no group is left open (1406.5453,
/// 1605.05104).
#[test]
fn lstinline_robust_in_ifthen_equal() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/lstinline_robust_in_ifthen_equal.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">full</p></para>"#,
  );
}

/// E5: a brace-less `\abstract` ended by the enclosing `\end{@twocolumnfalse}`: its capture frame is a bare
/// environment's, so the `\endgroup` ends the abstract first (an Info, TeX has no mismatch) and the section follows
/// it (2305.17823).
#[test]
fn bare_abstract_ended_by_enclosing_endgroup() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/bare_abstract_ended_by_enclosing_endgroup.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    log.matches("inserting \\lx@end@abstract").count(),
    1,
    "{log}"
  );
  assert_eq!(xml.matches("<abstract").count(), 1, "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">The abstract text.</p></abstract>"#,
  );
  let abstract_at = xml.find("<abstract").unwrap();
  let section_at = xml
    .find("<section")
    .unwrap_or_else(|| panic!("no section: {xml}"));
  assert!(abstract_at < section_at, "{xml}");
  assert!(xml.contains(r#"<p xml:id="S1.p1.1">Body.</p>"#), "{xml}");
}

/// E6: IEEEtran.cls has no `\keywords`; the binding's convenience one is a fallback, so the document's
/// `\newcommand{\keywords}{…}` wins and `{IEEEkeywords}` holds its text (2006.10005, 2008.05854, 2011.04315).
#[test]
fn ieeetran_document_keywords_macro_wins() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_document_keywords_macro_wins.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<keywords").count(), 1, "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Index Terms:\u{a0}\">M-estimators, shrinkage, regularization</keywords>",
  );
}

/// E6, the fallback still works: without a document definition, IEEEtran's `\keywords{…}` and the
/// `\keywords … \endkeywords` form are the keywords block.
#[test]
fn ieeetran_keywords_fallback_still_works() {
  let tex = "\\documentclass{IEEEtran}\n\\begin{document}\n\\title{T}\n\\maketitle\n\\keywords{alpha, beta}\n\
             \\section{Intro}\nText.\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Index Terms:\u{a0}\">alpha, beta </keywords>",
  );
}

/// E8: AASTeX 6+ have no `{plate}`, so tikz bayesnet's `\newcommand{\plate}` is the paper's (1909.00812, 2006.16258,
/// 2305.06275); AASTeX 5's `{plate}` stays.
#[test]
fn aastex6_plus_has_no_plate() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aastex6_plus_has_no_plate.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("ltx_float_plate"), "{xml}");
  assert_eq!(xml.matches("<picture").count(), 1, "{xml}");
  assert!(xml.contains(r#"tex="N""#), "{xml}");
}

#[test]
fn aastex5_keeps_its_plate() {
  let tex = "\\documentclass{aastex}\n\\begin{document}\n\\begin{plate}\nX\n\\caption{A plate.}\n\\end{plate}\n\
             \\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(
    xml.matches(r#"class="ltx_float_plate""#).count(),
    1,
    "{xml}"
  );
}

/// E9: emulateapj's deluxetable ended without `\enddata` (a longtable there, emulateapj.cls:1430-1460, :1497): its
/// end closes the open data (1506.07185, 1705.08932).
#[test]
fn emulateapj_deluxetable_without_enddata() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/emulateapj_deluxetable_without_enddata.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<table ").count(), 1, "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "tbody",
    &[],
    r#"<tbody><tr xml:id="S0.T1.2.2"><td align="center" border="t" xml:id="S0.T1.2.2.1">a</td><td align="center" border="t" xml:id="S0.T1.2.2.2">b</td></tr><tr xml:id="S0.T1.2.3"><td align="center" border="b" xml:id="S0.T1.2.3.1">c</td><td align="center" border="b" xml:id="S0.T1.2.3.2">d</td></tr></tbody>"#,
  );
}

/// E1, the binding-made half: algorithm2e.sty loads no float package, so its binding no longer loads float's beside
/// floatrow — before floatrow (whose `\newcommand`s float's binding refused, 1807.05059) or after it (1811.04370,
/// 1609.05867, 1910.09976) — and floatrow's `\float@end` closes the restyled float.
#[test]
fn algorithm2e_beside_floatrow_keeps_floatrows_end() {
  let before = include_str!(
    "../../../tools/perfect_kernel/repros/loader/algorithm2e_beside_floatrow_keeps_floatrows_end.tex"
  );
  let after = before.replace(
    "\\usepackage{algorithm2e}\n\\usepackage{floatrow}",
    "\\usepackage{floatrow}\n\\usepackage{algorithm2e}",
  );
  assert_ne!(before, after);
  for tex in [before, after.as_str()] {
    let (log, xml) = convert_counted(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert!(!log.contains("(Loading float_sty.rs"), "{log}");
    assert_eq!(xml.matches("<caption>").count(), 1, "{xml}");
    let float = xml
      .split(r#"<float class="ltx_float_figure""#)
      .nth(1)
      .and_then(|rest| rest.split("</float>").next())
      .unwrap_or_else(|| panic!("no figure float: {xml}"));
    assert!(
      float.contains(r#"<caption><tag close=": ">Figure 1</tag>A caption.</caption>"#),
      "{xml}"
    );
  }
}
