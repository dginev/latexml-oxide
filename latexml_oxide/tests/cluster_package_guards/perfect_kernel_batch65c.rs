//! Red/green guards for perfect-kernel batch 65c: environments left open at `\end{document}` (and inside a runaway
//! `\abstract{…` ended by a section) keep their content. The fixtures are ill-formed on purpose — pdflatex reports the
//! open environment ("\begin{X} … ended by \end{document}") and prints every word — so each pins its exact diagnostic
//! counts instead of requiring a clean conversion.
use latexml::util::test::{assert_element, convert_with, rng_error_count, warning_count};

use super::perfect_kernel_batch46::error_count;

/// The (log, XML) of a `boxes-groups` repro converted under ar5iv, with exactly `errors` errors and `warnings`
/// warnings, and schema-valid (when jing is installed).
fn convert_counted(tex: &str, errors: usize, warnings: usize) -> (String, String) {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), errors, "{log}");
  assert_eq!(warning_count(&log), warnings, "{log}");
  if let Some(invalid) = rng_error_count(&xml) {
    assert_eq!(invalid, 0, "schema errors:\n{xml}");
  }
  (log, xml)
}

/// The one warning of an `\end{document}` met with environments still open: Perl's text, one detail line per open
/// environment, innermost first (latex_constructs.pool.ltxml:364-379).
fn assert_open_environment_lines(log: &str, lines: &[&str]) {
  assert!(
    log.contains("Warning:unexpected:\\end{document} Attempt to end document with open groups, environments or conditionals"),
    "{log}"
  );
  let details: Vec<&str> = log
    .lines()
    .filter_map(|line| line.strip_prefix('\t'))
    .filter(|line| {
      line.starts_with("Environment ")
        || line.starts_with("Conditional ")
        || line.starts_with("Group ")
    })
    .collect();
  assert_eq!(details.len(), lines.len(), "{log}");
  for (detail, prefix) in details.iter().zip(lines) {
    assert!(
      detail.starts_with(prefix),
      "{detail:?} does not start with {prefix:?}\n{log}"
    );
    assert!(!detail.contains("Stored::"), "{log}");
  }
}

/// 65c: two quotes open at `\end{document}`, the inner inside the outer. Every body still digesting at the
/// end-of-document unwinding ends as at end of input (stomach.rs `abandon_stack_frame`; Perl Stomach.pm:111,130), so
/// the outer body no longer hands the whole inner quote to `set_body` as its trailer. Repro
/// boxes-groups/unclosed_nested_quotes_at_end_document.
#[test]
fn unclosed_nested_quotes_at_end_document() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/unclosed_nested_quotes_at_end_document.tex"
  );
  let (log, xml) = convert_counted(tex, 0, 1);
  assert_element(
    &xml,
    "quote",
    &["xml:id=\"p1.2\""],
    "<quote xml:id=\"p1.2\">\n      <p xml:id=\"p1.2.1\">Q1.</p>\n      <quote xml:id=\"p1.2.2\">\n        <p xml:id=\"p1.2.2.1\">Q2. After.</p>\n      </quote>\n    </quote>",
  );
  assert_open_environment_lines(&log, &[
    "Environment quote opened by \\begin{quote} ",
    "Environment quote opened by \\begin{quote} ",
  ]);
}

/// 65c: an `{itemize}` open inside an open `{quote}` at `\end{document}` stays in the quote, its item whole. Repro
/// boxes-groups/unclosed_quote_around_itemize_at_end_document.
#[test]
fn unclosed_quote_around_itemize_at_end_document() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/unclosed_quote_around_itemize_at_end_document.tex"
  );
  let (log, xml) = convert_counted(tex, 0, 1);
  assert_element(
    &xml,
    "quote",
    &["xml:id=\"p1.2\""],
    "<quote xml:id=\"p1.2\">\n      <p xml:id=\"p1.2.1\">Q1.</p>\n      <itemize xml:id=\"S0.I1\">\n        <item xml:id=\"S0.I1.i1\">\n          <tags>\n            <tag>•</tag>\n            <tag role=\"typerefnum\">1st item</tag>\n          </tags>\n          <para xml:id=\"S0.I1.i1.p1\">\n            <p xml:id=\"S0.I1.i1.p1.1\">I2. After.</p>\n          </para>\n        </item>\n      </itemize>\n    </quote>",
  );
  assert_open_environment_lines(&log, &[
    "Environment itemize opened by \\begin{itemize} ",
    "Environment quote opened by \\begin{quote} ",
  ]);
}

/// 65c: a second-level `{itemize}` open inside an open item at `\end{document}` stays in that item. Repro
/// boxes-groups/unclosed_nested_itemize_at_end_document.
#[test]
fn unclosed_nested_itemize_at_end_document() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/unclosed_nested_itemize_at_end_document.tex"
  );
  let (log, xml) = convert_counted(tex, 0, 1);
  assert_element(
    &xml,
    "item",
    &["xml:id=\"S0.I1.i1\""],
    "<item xml:id=\"S0.I1.i1\">\n        <tags>\n          <tag>•</tag>\n          <tag role=\"typerefnum\">1st item</tag>\n        </tags>\n        <para xml:id=\"S0.I1.i1.p1\">\n          <p xml:id=\"S0.I1.i1.p1.1\">I1.</p>\n          <itemize xml:id=\"S0.I1.i1.I1\">\n            <item xml:id=\"S0.I1.i1.I1.i1\">\n              <tags>\n                <tag><text font=\"bold\" xml:id=\"S0.I1.i1.I1.i1.1\">–</text></tag>\n                <tag role=\"typerefnum\">1st item</tag>\n              </tags>\n              <para xml:id=\"S0.I1.i1.I1.i1.p1\">\n                <p xml:id=\"S0.I1.i1.I1.i1.p1.1\">I2. After.</p>\n              </para>\n            </item>\n          </itemize>\n        </para>\n      </item>",
  );
  assert_open_environment_lines(&log, &[
    "Environment itemize opened by \\begin{itemize} ",
    "Environment itemize opened by \\begin{itemize} ",
  ]);
}

/// 65c: the 64zf review shape — each `{quote}`'s `\end{quote}` closes a `\begingroup` instead (the two errors), so
/// both quotes are still open, one inside the other, at `\end{document}`; the second keeps "Q2." and "After.". Repro
/// boxes-groups/unclosed_nested_environments_keep_their_text.
#[test]
fn unclosed_nested_environments_keep_their_text() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/unclosed_nested_environments_keep_their_text.tex"
  );
  let (_log, xml) = convert_counted(tex, 2, 1);
  assert_element(
    &xml,
    "quote",
    &["xml:id=\"p1.2\""],
    "<quote xml:id=\"p1.2\">\n      <p xml:id=\"p1.2.1\">Q1.\nMiddle.</p>\n      <quote xml:id=\"p1.2.2\">\n        <p xml:id=\"p1.2.2.1\">Q2.\nAfter.</p>\n      </quote>\n    </quote>",
  );
}

/// 65c: conditionals still open at `\end{document}` are closed and reported after the environments, as Perl does
/// (latex_constructs.pool.ltxml:374-377; tex.web §1335 "(\end occurred when \iftrue on line N was incomplete)"); the
/// initiator prints as its token, not a `Stored::` debug form. Repro
/// boxes-groups/open_conditional_reported_at_end_document.
#[test]
fn open_conditional_reported_at_end_document() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/open_conditional_reported_at_end_document.tex"
  );
  let (log, xml) = convert_counted(tex, 0, 1);
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">Before. Inside.</p>\n    <quote xml:id=\"p1.2\">\n      <p xml:id=\"p1.2.1\">Q1.</p>\n      <quote xml:id=\"p1.2.2\">\n        <p xml:id=\"p1.2.2.1\">Q2. After.</p>\n      </quote>\n    </quote>\n  </para>",
  );
  assert_open_environment_lines(&log, &[
    "Environment quote opened by \\begin{quote} ",
    "Environment quote opened by \\begin{quote} ",
    "Conditional \\iftrue started ",
  ]);
}

/// 65c: the sibling unwinding — a runaway `\abstract{…` ended by a section (`until_terminal_inside_group`,
/// screenplay-pkg.tex:67) abandons the quotes opened inside it; both stay in the abstract, the inner one whole, and
/// the section lands at document level. Repro boxes-groups/unclosed_quotes_in_runaway_abstract_keep_their_text.
#[test]
fn unclosed_quotes_in_runaway_abstract_keep_their_text() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/unclosed_quotes_in_runaway_abstract_keep_their_text.tex"
  );
  let (log, xml) = convert_counted(tex, 0, 1);
  assert!(
    log.contains("Warning:unexpected:\\lx@end@abstract"),
    "{log}"
  );
  assert_element(
    &xml,
    "abstract",
    &["xml:id=\"abstract1\""],
    "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\">\n    <p xml:id=\"abstract1.1\">Abstract one.</p>\n    <quote xml:id=\"abstract1.2\">\n      <p xml:id=\"abstract1.2.1\">Quote two.</p>\n      <quote xml:id=\"abstract1.2.2\">\n        <p xml:id=\"abstract1.2.2.1\">Quote three.</p>\n      </quote>\n    </quote>\n  </abstract>",
  );
  assert_element(
    &xml,
    "section",
    &["xml:id=\"S1\""],
    "<section inlist=\"toc\" xml:id=\"S1\">\n    <tags>\n      <tag>1</tag>\n      <tag role=\"refnum\">1</tag>\n      <tag role=\"typerefnum\">§1</tag>\n    </tags>\n    <title><tag close=\" \">1</tag>Intro</title>\n    <para xml:id=\"S1.p1\">\n      <p xml:id=\"S1.p1.1\">Following four.</p>\n    </para>\n  </section>",
  );
  // The section follows the abstract at document level, not inside it.
  assert!(
    xml.contains("</abstract>\n  <section inlist=\"toc\" xml:id=\"S1\">"),
    "{xml}"
  );
}

/// 65c r2: an environment that captures no body (`\newenvironment{myenv}{\small}{}`) left open at `\end{document}`
/// leaves the bare-`\bibitem` bibliography open; the `ltx:document` close, deferred to the end of the build
/// (core_interface.rs `close_abandoned_document`), closes it with its close-time hooks, so the bold "References"
/// paragraph becomes the bibliography's title as in a closed document (review r1 probe v1: skipping the close printed
/// it twice). Repro boxes-groups/abandoned_document_close_runs_bibliography_hooks.
#[test]
fn abandoned_document_close_runs_bibliography_hooks() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/abandoned_document_close_runs_bibliography_hooks.tex"
  );
  let (log, xml) = convert_counted(tex, 0, 1);
  assert_open_environment_lines(&log, &["Environment myenv opened by \\begingroup "]);
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"bib\""],
    "<bibliography inlist=\"toc\" xml:id=\"bib\">\n    <title fontsize=\"90%\">References</title>\n    <biblist>\n      <bibitem key=\"a\" xml:id=\"bib.bib1\">\n        <tags>\n          <tag>[1]</tag>\n          <tag role=\"refnum\">1</tag>\n        </tags>\n        <bibblock><text fontsize=\"90%\" xml:id=\"bib.bib1.1\"> A ref.\n</text></bibblock>\n      </bibitem>\n      <bibitem key=\"b\" xml:id=\"bib.bib2\">\n        <tags>\n          <tag>[2]</tag>\n          <tag role=\"refnum\">2</tag>\n        </tags>\n        <bibblock><text fontsize=\"90%\" xml:id=\"bib.bib2.1\"> B ref.\n</text></bibblock>\n      </bibitem>\n    </biblist>\n  </bibliography>",
  );
  assert_eq!(xml.matches("References").count(), 1, "{xml}");
}

/// 65c r2: the same shape without a heading — the deferred close runs the bibliography hooks, which add no title the
/// source does not print (review r1 probe r11: skipping the close invented `<title>References</title>` +
/// `inlist="toc"`). Repro boxes-groups/abandoned_document_close_invents_no_bibliography_title.
#[test]
fn abandoned_document_close_invents_no_bibliography_title() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/abandoned_document_close_invents_no_bibliography_title.tex"
  );
  let (_log, xml) = convert_counted(tex, 0, 1);
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"bib\""],
    "<bibliography xml:id=\"bib\">\n    <biblist>\n      <bibitem key=\"a\" xml:id=\"bib.bib1\">\n        <tags>\n          <tag>[1]</tag>\n          <tag role=\"refnum\">1</tag>\n        </tags>\n        <bibblock><text font=\"italic\" xml:id=\"bib.bib1.1\"> A ref.\n</text></bibblock>\n      </bibitem>\n      <bibitem key=\"b\" xml:id=\"bib.bib2\">\n        <tags>\n          <tag>[2]</tag>\n          <tag role=\"refnum\">2</tag>\n        </tags>\n        <bibblock><text font=\"italic\" xml:id=\"bib.bib2.1\"> B ref.\n</text></bibblock>\n      </bibitem>\n    </biblist>\n  </bibliography>",
  );
}

/// 65c r2: a math left open to `\end{document}` reverts its whole body into `tex=`; the end-of-document internals
/// (`\lx@frontmatter@fallback`, `\lx@finalize@document`) revert to nothing. Since 65n the end's `\par` ends the inline
/// formula ("Missing $ inserted", tex.web §1046-1047, = pdflatex; OXIDIZED_DESIGN #553), so the `tex=` ends with its
/// text. Repro boxes-groups/math_open_at_end_document_reverts_no_internal_names.
#[test]
fn math_open_at_end_document_reverts_no_internal_names() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/math_open_at_end_document_reverts_no_internal_names.tex"
  );
  let (log, xml) = convert_counted(tex, 1, 0);
  assert!(log.contains("Error:expected:$ Missing $ inserted"), "{log}");
  assert!(
    xml.contains("<Math mode=\"inline\" tex=\"x+yTailwords.\" text=\"x + y * T * a * i * l * w * o * r * d * s\" xml:id=\"p1.m1\">"),
    "{xml}"
  );
  assert!(!xml.contains("\\lx@"), "{xml}");
}
