//! Red/green guards for the int19 fix round (the bundle A/B's worse papers, agent_reports/2026-10-17_payload_int19.md).
//! Item 1 — the token after a display's closing `$` is read in the display's own mode again (69b r2's enclosing-mode
//! read reverted: with Perl's "Ignoring" recovery it cascaded, arXiv 2207.12569). Item 2 — amstext's `\text` digests
//! its argument inside box braces (amstext.sty:28-39), so an alignment's `\cr` in it never ends the cell around it
//! (arXiv 1703.08664).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{convert_with, error_count, warning_count};

/// expansion-primitives/display_closing_dollar_text_after_the_display_kept: arXiv 2207.12569's
/// `$${\boxed{\mbox{…}}}\Longrightarrow{\boxed{$u$\mbox{-}$S$\mbox{-Artinian rings}}}…$$` (pdflatex 0 errors). 69b r2
/// read the token after each `$` in `\boxed` under the enclosing mode, as text, then re-read it in the display Perl's
/// recovery leaves open: 8 errors here, 6 -> 101 + Fatal on the paper, the paragraph after the display lost. Base's
/// count again (the six `$` in `\boxed`), the display closed by its `$$`, the paragraph after it kept.
#[test]
fn display_closing_dollar_text_after_the_display_kept() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/display_closing_dollar_text_after_the_display_kept.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 6, "{stderr}");
  assert_eq!(
    stderr
      .matches("Error:expected:$ Missing $ closing display math.")
      .count(),
    6,
    "{stderr}"
  );
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>We have the implications</p>"), "{xml}");
  assert!(
    xml.contains("<p>But neither of implications can be reversed.</p>"),
    "{xml}"
  );
  assert_eq!(xml.matches("<Math mode=\"display\"").count(), 1, "{xml}");
}
