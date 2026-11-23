//! Guards for ACMTEASER: acmart's `teaserfigure` stands where `\maketitle` prints it, after the authors and before the
//! abstract (acmart.cls:2240 `\@mktitle\@mkauthors\@mkteasers`, then :2331 the abstract and keywords; sigchi-a after
//! the title), in source order, inside one continued title page (`ltx_titlepage_continued`: the top-matter box
//! `\maketitle` prints it in, DG 2026-11-23), keeping its number and label (OXIDIZED_DESIGN_DIVERGENCES #1146,
//! KNOWN_PERL_ERRORS #1319: Perl leaves it ahead of the title).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{convert_with, error_count, rng_error_count, warning_count};

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

/// One row per placement path: the repro, the document's top-level elements in order, the continued title page whole
/// (`~` for the autoref's no-break space).
const ROWS: &[(&str, &str, &[&str], &str)] = &[
  // Two teasers written after the abstract and keywords, before `\maketitle` (the corpus shape: 2606.22880). Before
  // (INT081C): both after the keywords; round 1 of ACMTEASER: both bare between the authors and the abstract.
  (
    "acmart_teaserfigure_after_authors",
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/acmart_teaserfigure_after_authors.tex"
    ),
    &[
      "<title>",
      r#"<creator role="author">"#,
      r#"<creator before=" and " role="author">"#,
      r#"<titlepage class="ltx_titlepage_continued">"#,
      r#"<abstract inlist="toc" name="Abstract." xml:id="abstract1">"#,
      "<keywords name=\"Keywords:\u{a0}\">",
      r#"<section inlist="toc" xml:id="S1">"#,
    ],
    r#"<titlepage class="ltx_titlepage_continued">
    <figure class="ltx_teaserfigure" inlist="lof" labels="LABEL:fig:t1" xml:id="S0.F1">
      <tags>
        <tag>Figure 1</tag>
        <tag role="autoref">Figure~1</tag>
        <tag role="refnum">1</tag>
        <tag role="typerefnum">Figure 1</tag>
      </tags>
      <rule align="center" height="28.5pt" width="56.9pt"/>
      <toccaption class="ltx_centering"><tag close=" ">1</tag>First teaser</toccaption>
      <caption class="ltx_centering"><tag close=": ">Figure 1</tag>First teaser</caption>
    </figure>
    <figure class="ltx_teaserfigure" inlist="lof" labels="LABEL:fig:t2" xml:id="S0.F2">
      <tags>
        <tag>Figure 2</tag>
        <tag role="autoref">Figure~2</tag>
        <tag role="refnum">2</tag>
        <tag role="typerefnum">Figure 2</tag>
      </tags>
      <rule align="center" height="28.5pt" width="56.9pt"/>
      <toccaption class="ltx_centering"><tag close=" ">2</tag>Second teaser</toccaption>
      <caption class="ltx_centering"><tag close=": ">Figure 2</tag>Second teaser</caption>
    </figure>
  </titlepage>"#,
  ),
  // sigchi-a prints the teasers between the title and the authors (`\if@ACM@sigchiamode`). Before: after the abstract.
  (
    "acmart_teaserfigure_sigchia_after_title",
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/acmart_teaserfigure_sigchia_after_title.tex"
    ),
    &[
      r#"<title font="sansserif">"#,
      r#"<titlepage class="ltx_titlepage_continued">"#,
      r#"<creator role="author">"#,
      r#"<abstract inlist="toc" name="Abstract." xml:id="abstract1">"#,
      r#"<section inlist="toc" xml:id="S1">"#,
    ],
    r#"<titlepage class="ltx_titlepage_continued">
    <figure class="ltx_teaserfigure" inlist="lof" labels="LABEL:fig:t1" xml:id="S0.F1">
      <tags>
        <tag><text font="sansserif">Figure 1</text></tag>
        <tag role="autoref"><text font="sansserif">Figure~1</text></tag>
        <tag role="refnum"><text font="sansserif">1</text></tag>
        <tag role="typerefnum"><text font="sansserif">Figure 1</text></tag>
      </tags>
      <rule align="center" height="28.5pt" width="56.9pt"/>
      <toccaption class="ltx_centering"><tag close=" "><text font="sansserif">1</text></tag><text font="sansserif">Chi teaser</text></toccaption>
      <caption class="ltx_centering" font="sansserif"><tag close=": ">Figure 1</tag>Chi teaser</caption>
    </figure>
  </titlepage>"#,
  ),
  // No `\maketitle`, a second teaser inside the section: the placement happens at the document's close, whatever wrote
  // the frontmatter, and takes every teaser into the one wrapper (it stands among the frontmatter all the same). Before:
  // the first after the abstract, the second in the section.
  (
    "acmart_teaserfigure_without_maketitle",
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/acmart_teaserfigure_without_maketitle.tex"
    ),
    &[
      "<title>",
      r#"<creator role="author">"#,
      r#"<titlepage class="ltx_titlepage_continued">"#,
      r#"<abstract inlist="toc" name="Abstract." xml:id="abstract1">"#,
      r#"<section inlist="toc" xml:id="S1">"#,
    ],
    r#"<titlepage class="ltx_titlepage_continued">
    <figure class="ltx_teaserfigure" inlist="lof" labels="LABEL:fig:t1" xml:id="S0.F1">
      <tags>
        <tag>Figure 1</tag>
        <tag role="autoref">Figure~1</tag>
        <tag role="refnum">1</tag>
        <tag role="typerefnum">Figure 1</tag>
      </tags>
      <rule align="center" height="28.5pt" width="56.9pt"/>
      <toccaption class="ltx_centering"><tag close=" ">1</tag>Lost teaser</toccaption>
      <caption class="ltx_centering"><tag close=": ">Figure 1</tag>Lost teaser</caption>
    </figure>
    <figure class="ltx_teaserfigure" inlist="lof" labels="LABEL:fig:t2" xml:id="S1.F2">
      <tags>
        <tag>Figure 2</tag>
        <tag role="autoref">Figure~2</tag>
        <tag role="refnum">2</tag>
        <tag role="typerefnum">Figure 2</tag>
      </tags>
      <rule align="center" height="28.5pt" width="56.9pt"/>
      <toccaption class="ltx_centering"><tag close=" ">2</tag>Late teaser</toccaption>
      <caption class="ltx_centering"><tag close=": ">Figure 2</tag>Late teaser</caption>
    </figure>
  </titlepage>"#,
  ),
];

/// class-bindings/acmart_teaserfigure_*: each converts with 0 Errors and 0 Warnings, its top-level elements exactly in
/// acmart's printed order, every teaser once, inside the one continued title page whole, the body's `\ref`s naming
/// the teasers' labels, and the document valid against the RelaxNG schema.
#[test]
fn acmart_teaserfigure_stands_where_maketitle_prints_it() {
  for (name, tex, order, wrapper) in ROWS {
    let (log, xml) = convert_with(tex, None);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}:\n{log}"
    );
    assert_eq!(top_level(&xml), *order, "{name}:\n{xml}");
    let at = xml
      .find("<titlepage ")
      .unwrap_or_else(|| panic!("{name}: no titlepage\n{xml}"));
    let end = at + xml[at..].find("</titlepage>").expect("closed") + "</titlepage>".len();
    assert_eq!(
      xml[at..end],
      wrapper.replace('~', "\u{a0}"),
      "{name}:\n{xml}"
    );
    assert_eq!(
      xml.matches("ltx_teaserfigure").count(),
      wrapper.matches("ltx_teaserfigure").count(),
      "{name}:\n{xml}"
    );
    assert!(
      xml.contains(r#"labelref="LABEL:fig:t1"/>"#),
      "{name}:\n{xml}"
    );
    // (a titlepage admits a figure: SCHEMA_EXTENSIONS S14; skipped visibly without jing)
    if let Some(n) = rng_error_count(&xml) {
      assert_eq!(n, 0, "{name}: schema-invalid\n{xml}");
    }
  }
}

/// sectioning-frontmatter/titlepage_env_keeps_its_float: the other path SCHEMA_EXTENSIONS S14 opens, the builder's
/// (the model's `canContain`, not a close hook's move): a float written inside `\begin{titlepage}` stays in it, with
/// the cover text after it, as the title page prints them; valid against the schema. Before: the titlepage ended at
/// the figure (Perl's `autoClose`) and "More cover." joined the body.
#[test]
fn a_float_in_a_titlepage_stays_in_it() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlepage_env_keeps_its_float.tex"
    ),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let at = xml.find("<titlepage").expect("titlepage");
  let end = at + xml[at..].find("</titlepage>").expect("closed") + "</titlepage>".len();
  let expected = r#"<titlepage>
    <p>Cover text.</p>
    <figure inlist="lof" labels="LABEL:f" placement="h" xml:id="S0.F1">
      <tags>
        <tag>Figure 1</tag>
        <tag role="refnum">1</tag>
        <tag role="typerefnum">Figure 1</tag>
      </tags>
      <p>X</p>
      <toccaption><tag close=" ">1</tag>In cover</toccaption>
      <caption><tag close=": ">Figure 1</tag>In cover</caption>
    </figure>
    <p>More cover.</p>
  </titlepage>"#;
  assert_eq!(&xml[at..end], expected, "{xml}");
  assert!(xml.contains(r#"<ref labelref="LABEL:f"/>"#), "{xml}");
  if let Some(n) = rng_error_count(&xml) {
    assert_eq!(n, 0, "schema-invalid\n{xml}");
  }
}
