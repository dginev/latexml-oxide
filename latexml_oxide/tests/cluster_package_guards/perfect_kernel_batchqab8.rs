//! Guards for QAB8 (QA26): the glue between two sibling paragraphs is kept on the second as
//! `cssstyle="--ltx-skip-before:<pt>"` (tex_glue.rs `note_skip_after_para`, tex_paragraph.rs `apply_skip_before`;
//! OXIDIZED_DESIGN_DIVERGENCES #1050). Witnesses 2503.04888, 2001.10915, 2408.07944, 1606.03039.
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{convert_with, error_count, warning_count};

/// Each repro with every paragraph's start tag, in order: the carried skips and the paragraphs left bare.
/// skip_between_paragraphs_goes_on_the_next: `\bigskip`, `\medskip\noindent`, `\vspace{1cm}` (the QA26 min).
/// skip_between_paragraphs_shapes: no skip kept after or before a heading, for a negative skip, in a figure (its
/// panels are `ltx:p`), after a theorem, in a minipage, or for a `\vspace` inside a paragraph (`\vadjust`); kept
/// for a `\medskip` read in a paragraph a `\par` ends at once, `\bigskip\medskip` summed, a `\smallskip`, a `\vskip`
/// read in a paragraph (which ends it), and past an empty `\noindent` paragraph that is pruned; a net, as on TeX's
/// vertical list (`\bigskip\vspace{-6pt}` 6pt, `\vskip 36pt\vskip -36pt` none: needspace's shape).
/// skip_moved_into_a_title_page_is_dropped: cover paragraphs the frontmatter moves into a title page keep no skip.
/// skip_in_a_minipage_is_dropped: nor do a box's paragraphs, built in `insert_block`'s capture (2512.08710).
#[test]
fn paragraph_skips_go_on_the_next_paragraph() {
  let cases: [(&str, &[&str]); 4] = [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/paragraph-mode/skip_between_paragraphs_goes_on_the_next.tex"
      ),
      &[
        r#"<para xml:id="S1.p1">"#,
        r#"<para xml:id="S1.p2">"#,
        r#"<para cssstyle="--ltx-skip-before:12.0pt" xml:id="S1.p3">"#,
        r#"<para class="ltx_noindent" cssstyle="--ltx-skip-before:6.0pt" xml:id="S1.p4">"#,
        r#"<para cssstyle="--ltx-skip-before:28.5pt" xml:id="S1.p5">"#,
      ],
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/paragraph-mode/skip_between_paragraphs_shapes.tex"
      ),
      &[
        r#"<para xml:id="S1.p1">"#,
        r#"<para xml:id="S1.p2">"#,
        r#"<para xml:id="S1.SS1.p1">"#,
        r#"<para xml:id="S1.SS1.p2">"#,
        r#"<para xml:id="S1.SS1.p3">"#,
        r#"<para xml:id="Thmtheorem1.p1">"#,
        r#"<para xml:id="S1.SS1.p4">"#,
        r#"<para class="ltx_noindent" xml:id="S1.SS1.p5">"#,
        r#"<para xml:id="S1.SS1.p6">"#,
        r#"<para xml:id="S1.SS1.p7">"#,
        r#"<para xml:id="S2.p1">"#,
        r#"<para xml:id="S2.p2">"#,
        r#"<para cssstyle="--ltx-skip-before:6.0pt" xml:id="S2.p3">"#,
        r#"<para cssstyle="--ltx-skip-before:18.0pt" xml:id="S2.p4">"#,
        r#"<para xml:id="S2.p5">"#,
        r#"<para cssstyle="--ltx-skip-before:3.0pt" xml:id="S2.p6">"#,
        r#"<para xml:id="S2.p7">"#,
        r#"<para cssstyle="--ltx-skip-before:28.5pt" xml:id="S2.p8">"#,
        r#"<para cssstyle="--ltx-skip-before:6.0pt" xml:id="S2.p9">"#,
        r#"<para cssstyle="--ltx-skip-before:6.0pt" xml:id="S2.p10">"#,
        r#"<para xml:id="S2.p11">"#,
      ],
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/paragraph-mode/skip_moved_into_a_title_page_is_dropped.tex"
      ),
      &[r#"<para xml:id="Ch1.p1">"#],
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/paragraph-mode/skip_in_a_minipage_is_dropped.tex"
      ),
      &[
        r#"<para class="ltx_noindent" xml:id="S1.p1">"#,
        r#"<para class="ltx_noindent" xml:id="S1.p1.p1">"#,
        r#"<para class="ltx_noindent" xml:id="S1.p1.p2">"#,
        r#"<para class="ltx_noindent" xml:id="S1.p1.p3">"#,
      ],
    ),
  ];
  for (tex, paras) in cases {
    let (stderr, xml) = convert_with(tex, None);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(warning_count(&stderr), 0, "{stderr}");
    let starts: Vec<&str> = xml
      .match_indices("<para ")
      .map(|(at, _)| &xml[at..at + xml[at..].find('>').unwrap() + 1])
      .collect();
    assert_eq!(starts, paras, "{xml}");
    // No other element carries a skip (the title page's blocks, 2007.05296), and no bookkeeping is left.
    let carried = paras
      .iter()
      .filter(|para| para.contains("--ltx-skip-before:"))
      .count();
    assert_eq!(xml.matches("--ltx-skip-before:").count(), carried, "{xml}");
    assert!(!xml.contains("_skip_"), "{xml}");
  }
}
