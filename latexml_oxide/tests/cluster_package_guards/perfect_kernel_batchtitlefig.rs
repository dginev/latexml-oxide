//! Guards for TITLEFIG: a float LaTeX prints in place inside the title block — `\captionof` (caption.sty, capt-of),
//! float.sty's `[H]`, a class's teaser — is the cover: between `\maketitle` and a late abstract it joins the continued
//! title page (`ltx_titlepage_continued`), ahead of the abstract, as the PDF prints it; a float that floats stays in the
//! body, a `\captionof` it holds included (`collapse_float` keeps the wrapper's in-place mark off it); vgtc's
//! teaser joins it from the class's own `\@maketitle` (vgtc_cls.rs) (DG 2026-11-23, OXIDIZED_DESIGN_DIVERGENCES #1149).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{convert_with, error_count, rng_error_count, warning_count};

/// The opening tag of every element directly under `<document>` but its `<resource>`s.
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

const TITLE: &str = "<title>";
const AUTHOR: &str = r#"<creator role="author">"#;
const COVER: &str = r#"<titlepage class="ltx_titlepage_continued">"#;
const ABSTRACT: &str = r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1">"#;
const SECTION: &str = r#"<section inlist="toc" xml:id="S1">"#;

/// The continued title page holding one figure whole: its opening tag, its body, its caption (`centred`: the
/// caption's `ltx_centering`).
fn cover_with(figure: &str, body: &str, caption: &str, centred: bool) -> String {
  let class = if centred {
    r#" class="ltx_centering""#
  } else {
    ""
  };
  format!(
    r#"<titlepage class="ltx_titlepage_continued">
    {figure}
      <tags>
        <tag>Figure 1</tag>
        <tag role="refnum">1</tag>
        <tag role="typerefnum">Figure 1</tag>
      </tags>
      {body}
      <toccaption{class}><tag close=" ">1</tag>{caption}</toccaption>
      <caption{class}><tag close=": ">Figure 1</tag>{caption}</caption>
    </figure>
  </titlepage>"#
  )
}

/// One row per path: the repro, the top-level elements in order, the continued title page whole (`None`: none).
fn rows() -> Vec<(
  &'static str,
  &'static str,
  Vec<&'static str>,
  Option<String>,
)> {
  vec![
    // caption.sty's `\captionof` wrapper (`\lx@float@inplace`), in a `{center}` inside `\twocolumn[{\maketitle …}]`:
    // the CVPR/ICCV idiom (2303.05007, 2312.02238)
    (
      "titlefig_cvpr_twocolumn_captionof",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlefig_cvpr_twocolumn_captionof.tex"
      ),
      vec![TITLE, AUTHOR, COVER, ABSTRACT, SECTION],
      Some(cover_with(
        r#"<figure align="center" inlist="lof" labels="LABEL:fig:teaser" xml:id="S0.F1">"#,
        r#"<rule height="28.5pt" width="113.8pt"/>"#,
        "The teaser.",
        false,
      )),
    ),
    // capt-of's `\def\@captype{figure}\caption`: the float a caption makes outside any float (sect09.rs), the vgtc
    // teaser's mechanism (2609.25700)
    (
      "titlefig_capt_of_caption_in_center",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlefig_capt_of_caption_in_center.tex"
      ),
      vec![TITLE, AUTHOR, COVER, ABSTRACT, SECTION],
      Some(cover_with(
        r#"<figure align="center" inlist="lof" labels="LABEL:fig:c" xml:id="S0.F1">"#,
        r#"<rule height="28.5pt" width="56.9pt"/>"#,
        "Capt-of teaser.",
        false,
      )),
    ),
    // float.sty's `[H]`, read from `placement`
    (
      "titlefig_float_here_after_maketitle",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlefig_float_here_after_maketitle.tex"
      ),
      vec![TITLE, AUTHOR, COVER, ABSTRACT, SECTION],
      Some(cover_with(
        r#"<figure inlist="lof" labels="LABEL:fig:teaser" placement="H" xml:id="S0.F1">"#,
        r#"<rule align="center" height="28.5pt" width="113.8pt"/>"#,
        "The teaser.",
        true,
      )),
    ),
    // a `\@maketitle` patch's `\noindent` minipage holding a `\captionof`: the paragraph gives way to its float in the
    // title page (`settle_float_paragraph`; 2609.35408)
    (
      "titlefig_maketitle_patch_minipage_captionof",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlefig_maketitle_patch_minipage_captionof.tex"
      ),
      vec![TITLE, AUTHOR, COVER, ABSTRACT, SECTION],
      Some(cover_with(
        r#"<figure align="center" class="ltx_minipage" inlist="lof" labels="LABEL:fig:teaser" vattach="middle" width="345.0pt" xml:id="S0.F1">"#,
        r#"<rule align="center" height="28.5pt" width="113.8pt"/>"#,
        "The teaser.",
        false,
      )),
    ),
    // vgtc's own `\@maketitle` prints the teaser above the abstract its `\maketitle` prints from its store: the binding
    // names the deposit's in-place figure and sets it after the authors (vgtc_cls.rs; 2609.25700)
    (
      "titlefig_class_maketitle_teaser_before_store_abstract",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlefig_class_maketitle_teaser_before_store_abstract.tex"
      ),
      vec![TITLE, AUTHOR, COVER, ABSTRACT, SECTION],
      Some(cover_with(
        r#"<figure align="center" class="ltx_minipage ltx_teaserfigure" inlist="lof" labels="LABEL:fig:teaser" vattach="middle" width="345.0pt" xml:id="S0.F1">"#,
        r#"<rule align="center" height="28.5pt" width="113.8pt"/>"#,
        "The teaser.",
        false,
      )),
    ),
    // a later late abstract (`place_later_abstract`): the figure between the two abstracts is its cover
    (
      "titlefig_inplace_figure_between_two_abstracts",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlefig_inplace_figure_between_two_abstracts.tex"
      ),
      vec![
        TITLE,
        AUTHOR,
        ABSTRACT,
        COVER,
        r#"<abstract inlist="toc" name="Abstract" xml:id="abstract2">"#,
        SECTION,
      ],
      Some(cover_with(
        r#"<figure align="center" inlist="lof" labels="LABEL:fig:b" xml:id="S0.F1">"#,
        r#"<rule height="28.5pt" width="56.9pt"/>"#,
        "Between.",
        false,
      )),
    ),
    // a float that floats (`[t]`) stays in the body, after the abstract
    (
      "titlefig_true_float_after_maketitle_stays",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlefig_true_float_after_maketitle_stays.tex"
      ),
      vec![
        TITLE,
        AUTHOR,
        ABSTRACT,
        r#"<figure inlist="lof" labels="LABEL:fig:float" placement="t" xml:id="S0.F1">"#,
        SECTION,
      ],
      None,
    ),
    // a floating figure holding a `\captionof` minipage: `collapse_float` folds the wrapper's figure in without its
    // in-place mark, the author's `[t]` decides (2312.04564, 2401.16468, 2510.26800)
    (
      "titlefig_floating_figure_minipage_captionof_stays",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlefig_floating_figure_minipage_captionof_stays.tex"
      ),
      vec![
        TITLE,
        AUTHOR,
        ABSTRACT,
        r#"<figure class="ltx_minipage" inlist="lof" labels="LABEL:fig:float" placement="t" vattach="middle" width="155.3pt" xml:id="S0.F1">"#,
        SECTION,
      ],
      None,
    ),
    // the same in a two-column `figure*[h]` (2510.09001 acl, 2609.21948 ieeeconf, 2510.17095)
    (
      "titlefig_floating_figure_star_captionof_stays",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlefig_floating_figure_star_captionof_stays.tex"
      ),
      vec![
        TITLE,
        AUTHOR,
        ABSTRACT,
        r#"<figure align="center" class="ltx_minipage" inlist="lof" labels="LABEL:fig:float" placement="h" vattach="middle" width="155.3pt" xml:id="S0.F1">"#,
        SECTION,
      ],
      None,
    ),
  ]
}

/// sectioning-frontmatter/titlefig_*: each converts with 0 Errors and 0 Warnings, its top-level elements in the PDF's
/// order, the continued title page whole (none for a float that floats), the figure's `\ref` resolved, no internal
/// `_inplace` left, and the document valid against the RelaxNG schema.
#[test]
fn an_in_place_figure_in_the_title_block_joins_the_continued_title_page() {
  for (name, tex, order, wrapper) in rows() {
    let (log, xml) = convert_with(tex, None);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}:\n{log}"
    );
    assert_eq!(top_level(&xml), order, "{name}:\n{xml}");
    match wrapper {
      Some(wrapper) => {
        let at = xml
          .find("<titlepage ")
          .unwrap_or_else(|| panic!("{name}: no titlepage\n{xml}"));
        let end = at + xml[at..].find("</titlepage>").expect("closed") + "</titlepage>".len();
        assert_eq!(xml[at..end], wrapper, "{name}:\n{xml}");
      },
      None => assert!(!xml.contains("<titlepage"), "{name}:\n{xml}"),
    }
    assert!(xml.contains("<ref labelref=\"LABEL:fig:"), "{name}:\n{xml}");
    assert!(!xml.contains("_inplace"), "{name}:\n{xml}");
    // (a titlepage admits a figure: SCHEMA_EXTENSIONS S14; skipped visibly without jing)
    if let Some(n) = rng_error_count(&xml) {
      assert_eq!(n, 0, "{name}: schema-invalid\n{xml}");
    }
  }
}
