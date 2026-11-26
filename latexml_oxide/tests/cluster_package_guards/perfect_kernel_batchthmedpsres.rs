//! Guards for THMEDPSRES (REGFIX-E's residuals, PR #960). One row per code path:
//! - thm-listof's filters under the thmtools binding (`thmtools_sty.rs` `listed_theorems`): the keys and commands that
//!   leave theorems out of `\listoftheorems` (2408.09583), and `\onlynamedtheorems`, read but listing every theorem of
//!   the set, with a Warning (KNOWN_PERL_ERRORS #1339).
//! - edpsmath's `\keywords`, which the class sets in a box only its title page prints (`edpsmath_cls.rs`): the
//!   frontmatter's keywords, with no footnote of them (1901.04804) and under `[proc]`, which prints none (1110.0287).
//! - amsart's default MSC edition (KNOWN_PERL_ERRORS #1337): the binding's `\subjclass` and a document's own store.
//! - an `\item[…]` label that steps its own list's counter (1704.06431, KNOWN_PERL_ERRORS #1338).
//!
//! Each converts its repro once and asserts the exact Error and Warning counts, the creators where the class has a
//! frontmatter, and every element of the kinds the change shows in, whole.

use latexml::util::test::{
  assert_all_elements, assert_no_element, convert_files_with, error_count, warning_count,
};

use super::perfect_kernel_batch84b::creators;

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

struct Row {
  name:     &'static str,
  tex:      &'static str,
  files:    &'static [(&'static str, &'static str)],
  counts:   (usize, usize),
  creators: &'static [&'static [&'static str]],
  /// Every element `tag` the output holds, in order.
  elements: &'static [(&'static str, &'static [&'static str])],
  /// Elements the output must not hold.
  absent:   &'static [&'static str],
}

const EDPS: &[(&str, &str)] = &[
  (
    "edpsmath.cls",
    repro!("sectioning-frontmatter/class_keywords/edpsmath.cls"),
  ),
  (
    "edpsproc.clo",
    repro!("sectioning-frontmatter/class_keywords/edpsproc.clo"),
  ),
];

const ROWS: &[Row] = &[
  Row {
    name:     "thmtools: ignoreall/show/ignore keys, \\ignoretheorems, \\showtheorems, \\setlisttheoremstyle",
    tex:      repro!("package-bindings/thmtools_listoftheorems_filters.tex"),
    files:    &[],
    counts:   (0, 0),
    creators: &[],
    elements: &[("TOC", &[
      r#"<TOC lists="theorem:model" scope="global"><title>List of Models</title></TOC>"#,
      r#"<TOC lists="theorem:theorem theorem:lemma" scope="global"><title>Main results</title></TOC>"#,
    ])],
    absent:   &[],
  },
  Row {
    name:     "thmtools: \\onlynamedtheorems lists the whole set, counted",
    tex:      repro!("package-bindings/thmtools_onlynamedtheorems_lists_named_only_red.tex"),
    files:    &[],
    counts:   (0, 1),
    creators: &[],
    elements: &[("TOC", &[
      r#"<TOC lists="thm" scope="global"><title>List of Theorems</title></TOC>"#,
    ])],
    absent:   &[],
  },
  Row {
    name:     "edpsmath: the keywords box its title page footnotes",
    tex:      repro!("sectioning-frontmatter/class_keywords/edpsmath_keywords_footnote.tex"),
    files:    EDPS,
    counts:   (0, 0),
    creators: &[&["A. Author"]],
    elements: &[("keywords", &[
      "<keywords name=\"Keywords and phrases.\u{a0}\">alpha, beta</keywords>",
    ])],
    absent:   &["note"],
  },
  Row {
    name:     "edpsmath [proc]: keywords the class prints none of",
    tex:      repro!("sectioning-frontmatter/class_keywords/edpsmath_keywords_proc.tex"),
    files:    EDPS,
    counts:   (0, 0),
    creators: &[&["A. Author"]],
    elements: &[("keywords", &[
      "<keywords name=\"Keywords and phrases.\u{a0}\">alpha, beta</keywords>",
    ])],
    absent:   &["note"],
  },
  Row {
    name:     "amsart: \\subjclass without a year after one with",
    tex:      repro!(
      "sectioning-frontmatter/amsart_subjclass_without_a_year_names_the_class_edition.tex"
    ),
    files:    &[],
    counts:   (0, 0),
    creators: &[&["A. Author"]],
    elements: &[("classification", &[
      "<classification name=\"2000 Mathematics Subject Classification:\u{a0}\" scheme=\"2000 Mathematics Subject Classification\">Primary 05C10</classification>",
      "<classification name=\"2020 Mathematics Subject Classification:\u{a0}\" scheme=\"2020 Mathematics Subject Classification\">Primary 31B05</classification>",
    ])],
    absent:   &[],
  },
  Row {
    name:     "amsart: a document's own \\@subjclass store",
    tex:      repro!("sectioning-frontmatter/amsart_document_subjclass_copy_default_edition.tex"),
    files:    &[],
    counts:   (0, 0),
    creators: &[&["A. Author"]],
    elements: &[("classification", &[
      "<classification name=\"2020 Mathematics Subject Classification:\u{a0}\" scheme=\"2020 Mathematics Subject Classification\">Primary 31B05</classification>",
    ])],
    absent:   &[],
  },
  Row {
    name:     "\\item[…] label stepping its own counter",
    tex:      repro!("lists/item_label_refsteps_its_own_counter.tex"),
    files:    &[],
    counts:   (0, 0),
    creators: &[],
    elements: &[
      ("item", &[
        r#"<item labels="LABEL:it:ma" xml:id="S0.I1.ix1"><tags><tag>MA)</tag><tag role="typerefnum">item MA)</tag></tags><para xml:id="S0.I1.ix1.p1"><p xml:id="S0.I1.ix1.p1.1">For some alpha.</p></para></item>"#,
      ]),
      ("ref", &[r#"<ref labelref="LABEL:it:ma"/>"#]),
    ],
    absent:   &[],
  },
];

/// Every row's repro converted once: its counts, creators and elements.
#[test]
fn thmedpsres_rows() {
  for row in ROWS {
    let (log, xml) = convert_files_with(row.tex, row.files, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      row.counts,
      "{}: Error and Warning counts\n{log}",
      row.name
    );
    let want: Vec<Vec<String>> = row
      .creators
      .iter()
      .map(|c| c.iter().map(|s| s.to_string()).collect())
      .collect();
    assert_eq!(creators(&xml), want, "{}: creators\n{xml}", row.name);
    for (tag, expected) in row.elements {
      assert_all_elements(&xml, tag, &[], expected);
    }
    for tag in row.absent {
      assert_no_element(&xml, tag, &[]);
    }
  }
}
