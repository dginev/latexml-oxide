//! VARW: a varwidth box is as wide as its widest line, at most the width given (varwidth.sty `\endvarwidth`;
//! `minipage_takes_natural_width`; KNOWN_PERL_ERRORS #1232, OD #1012). Witnesses algxpar-doc, 2401.12710.
use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// The `[\the\wd0]` rows of a repro, in order.
fn rows(xml: &str) -> Vec<&str> {
  xml
    .split("<p xml:id=\"")
    .skip(1)
    .filter_map(|p| p.split_once('>')?.1.split_once("</p>"))
    .map(|(text, _)| text)
    .collect()
}

/// One row per shape, pdflatex's widths (repro header): a line, two lines, a tabular, a display in text, a broken
/// paragraph, two paragraphs, `\centering`, a rule, a minipage inside, a varwidth in a minipage, nested varwidths, a bare
/// `\varwidth` in a group then a minipage (the flag taken by its own box), `[t]`, `\hspace`, empty, `\large`,
/// `\settowidth`, array's `V` column, a minipage (its given width); the box in the text keeps its given width.
/// Then algxpar's measured comment (algxpar-doc, 2401.12710): no overflow, one "⊳" per comment.
#[test]
fn varwidth_takes_its_widest_line() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/varwidth_measures_its_widest_line.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let widths = rows(&xml);
  assert_eq!(
    widths[..19],
    [
      "[22.3056pt]",
      "[86.19458pt]",
      "[44.00003pt]",
      "[41.62253pt]",
      "[80.02792pt]",
      "[115.13911pt]",
      "[22.3056pt]",
      "[142.26378pt]",
      "[113.81102pt]",
      "[170.71652pt]",
      "[10.5556pt]",
      "[79.2111pt]",
      "[45.58342pt]",
      "[62.1833pt]",
      "[0.0pt]",
      "[51.14253pt]",
      "[5.2778pt]",
      "[47.02785pt]",
      "[284.52756pt]",
    ],
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<inline-block class="ltx_minipage" vattach="middle" width="284.5pt" xml:id="p20.1.1">"#
    ),
    "{xml}"
  );

  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/listings-algorithms/algxpar_comment_width_measures_the_text.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches(">⊳</XMTok>").count(), 2, "{xml}");
  assert!(!xml.contains("width=\"-"), "{xml}");
  for comment in [
    "<text font=\"slanted\" xml:id=\"algx1.l1.2.2.1\">a comment</text>",
    "<text font=\"slanted\" xml:id=\"algx1.l2.3.1.1.1.2.1\">Select method to solve, select equations to solve and \
     more words here to make it long</text>",
  ] {
    assert!(xml.contains(comment), "{comment}: {xml}");
  }
  assert!(
    xml.contains(
      "<p class=\"ltx_align_left\" xml:id=\"algx1.l2.3.1.1\">a long statement text that goes on and on and on and on \
       and on and on and on and on and on and on\n"
    ),
    "{xml}"
  );
}
