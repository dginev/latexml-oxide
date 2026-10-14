//! Red/green guards for batch 67a: arXiv's build loads hyperref into every paper, so the ar5iv preload gives a paper
//! that never loaded url.sty or hyperref their `\url`/`\href` at `\begin{document}` — only the ones still undefined
//! (user ruling 2026-10-14, OXIDIZED_DESIGN #588).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// `\url` and `\href` with neither package loaded are hyperref's links, in the text and in the bibliography
/// (cond-mat0407333, cs0409028, hep-th0411273, 2101.07380).
#[test]
fn url_href_without_hyperref() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/url_href_without_hyperref.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">See <ref class="ltx_url" font="typewriter" href="http://example.org/a_b">http://example.org/a_b</ref> and <ref class="ltx_href" href="https://doi.org/10.1/x_y">the DOI</ref>.</p>"#,
  );
  assert!(
    xml.contains(r#"<bibblock> A. Author, <ref class="ltx_url" font="typewriter" href="http://arxiv.org/abs/1234.5678">http://arxiv.org/abs/1234.5678</ref>."#),
    "{xml}"
  );
}

/// Provide-only: a paper's own `\url` stays its own; only the missing `\href` is supplied.
#[test]
fn own_url_kept_missing_href_supplied() {
  let tex = "\\documentclass{article}\n\\newcommand{\\url}[1]{\\texttt{#1}}\n\\begin{document}\n\
             See \\url{example} and \\href{https://e.org}{there}.\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">See <text font="typewriter" xml:id="p1.1.1">example</text> and <ref class="ltx_href" href="https://e.org">there</ref>.</p>"#,
  );
}

/// Without the ar5iv preload nothing is supplied: `\url` stays undefined, as in Perl and pdflatex.
#[test]
fn no_fallback_without_the_preload() {
  let tex =
    "\\documentclass{article}\n\\begin{document}\nSee \\url{http://e.org}.\n\\end{document}\n";
  let (log, _xml) = latexml::util::test::convert_with(tex, None);
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(log.contains("Error:undefined:\\url"), "{log}");
}
