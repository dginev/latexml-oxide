//! Red/green guards for batch 66q: natbib's citation internals serve a document's own citation commands (K1 report
//! `agent_reports/2026-10-14_k1_error_increases.md`, row W2).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// `\citenum` over `\NAT@citexnum[][]` and `\citepos` over `\NAT@citetp` each give one citation of their key: the
/// bracketed bare-number form and the textual form (2104.01159, 2105.06268).
#[test]
fn natbib_internals_serve_user_cite_commands() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/natbib_internals_serve_user_cite_commands.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      r#"See [<cite class="ltx_citemacro_citep"><bibref bibrefs="gi_2004" separator="," show="Number" yyseparator=","/></cite>] and"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<cite class="ltx_citemacro_citet"><bibref bibrefs="omohundro_2008" separator="," show="Authors Phrase1NumberPhrase2" yyseparator=",">"#
    ),
    "{xml}"
  );
  assert!(xml.contains("</cite> thesis."), "{xml}");
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}
