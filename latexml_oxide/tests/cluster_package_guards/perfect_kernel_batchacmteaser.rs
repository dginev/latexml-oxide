//! Guards for ACMTEASER: acmart's `teaserfigure` stands where `\maketitle` prints it, after the authors and before the
//! abstract (acmart.cls:2240 `\@mktitle\@mkauthors\@mkteasers`, then :2331 the abstract and keywords; sigchi-a after
//! the title), in source order, keeping its number and label (OXIDIZED_DESIGN_DIVERGENCES #1146, KNOWN_PERL_ERRORS
//! #1319: Perl leaves it ahead of the title).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{convert_with, error_count, warning_count};

/// The opening tag of every element directly under `<document>` but its `<resource>`s (the pretty-printer indents
/// those by two spaces).
fn top_level(xml: &str) -> Vec<&str> {
  xml
    .lines()
    .filter(|l| l.starts_with("  <") && !l.starts_with("  </") && !l.starts_with("  <resource "))
    .map(|l| {
      let l = l.trim_start();
      &l[..=l.find('>').unwrap_or(l.len() - 1)]
    })
    .collect()
}

/// One row per placement path: the repro, the document's top-level elements in order, each teaser's caption.
const ROWS: &[(&str, &str, &[&str], &[&str])] = &[
  // Two teasers written after the abstract and keywords, before `\maketitle` (the corpus shape: 2606.22880). Before
  // (INT081C): both after the keywords.
  (
    "acmart_teaserfigure_after_authors",
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/acmart_teaserfigure_after_authors.tex"
    ),
    &[
      "<title>",
      r#"<creator role="author">"#,
      r#"<creator before=" and " role="author">"#,
      r#"<figure class="ltx_teaserfigure" inlist="lof" labels="LABEL:fig:t1" xml:id="S0.F1">"#,
      r#"<figure class="ltx_teaserfigure" inlist="lof" labels="LABEL:fig:t2" xml:id="S0.F2">"#,
      r#"<abstract inlist="toc" name="Abstract." xml:id="abstract1">"#,
      "<keywords name=\"Keywords:\u{a0}\">",
      r#"<section inlist="toc" xml:id="S1">"#,
    ],
    &[
      r#"<caption class="ltx_centering"><tag close=": ">Figure 1</tag>First teaser</caption>"#,
      r#"<caption class="ltx_centering"><tag close=": ">Figure 2</tag>Second teaser</caption>"#,
    ],
  ),
  // sigchi-a prints the teasers between the title and the authors (`\if@ACM@sigchiamode`). Before: after the abstract.
  (
    "acmart_teaserfigure_sigchia_after_title",
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/acmart_teaserfigure_sigchia_after_title.tex"
    ),
    &[
      r#"<title font="sansserif">"#,
      r#"<figure class="ltx_teaserfigure" inlist="lof" labels="LABEL:fig:t1" xml:id="S0.F1">"#,
      r#"<creator role="author">"#,
      r#"<abstract inlist="toc" name="Abstract." xml:id="abstract1">"#,
      r#"<section inlist="toc" xml:id="S1">"#,
    ],
    &[
      r#"<caption class="ltx_centering" font="sansserif"><tag close=": ">Figure 1</tag>Chi teaser</caption>"#,
    ],
  ),
  // No `\maketitle`, a second teaser inside the section: the placement happens at the document's close, whatever wrote
  // the frontmatter, and takes every teaser. Before: the first after the abstract, the second in the section.
  (
    "acmart_teaserfigure_without_maketitle",
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/acmart_teaserfigure_without_maketitle.tex"
    ),
    &[
      "<title>",
      r#"<creator role="author">"#,
      r#"<figure class="ltx_teaserfigure" inlist="lof" labels="LABEL:fig:t1" xml:id="S0.F1">"#,
      r#"<figure class="ltx_teaserfigure" inlist="lof" labels="LABEL:fig:t2" xml:id="S1.F2">"#,
      r#"<abstract inlist="toc" name="Abstract." xml:id="abstract1">"#,
      r#"<section inlist="toc" xml:id="S1">"#,
    ],
    &[
      r#"<caption class="ltx_centering"><tag close=": ">Figure 1</tag>Lost teaser</caption>"#,
      r#"<caption class="ltx_centering"><tag close=": ">Figure 2</tag>Late teaser</caption>"#,
    ],
  ),
];

/// class-bindings/acmart_teaserfigure_*: each converts with 0 Errors and 0 Warnings, its top-level elements exactly in
/// acmart's printed order, each teaser's caption whole, and the body's `\ref`s naming the teasers' labels.
#[test]
fn acmart_teaserfigure_stands_where_maketitle_prints_it() {
  for (name, tex, order, captions) in ROWS {
    let (log, xml) = convert_with(tex, None);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}:\n{log}"
    );
    assert_eq!(top_level(&xml), *order, "{name}:\n{xml}");
    for caption in *captions {
      assert_eq!(xml.matches(caption).count(), 1, "{name}: {caption}\n{xml}");
    }
    assert!(
      xml.contains(r#"labelref="LABEL:fig:t1"/>"#),
      "{name}:\n{xml}"
    );
  }
}
