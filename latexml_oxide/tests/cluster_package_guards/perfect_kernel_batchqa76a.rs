//! Guards for batch QA76A: a class's keyword and classification lists print their own `\and` (llncs.cls:906's
//! centred dot, svjour3.cls:304-322's `$\cdot$`; upstream brucemiller/LaTeXML#2263, KNOWN_PERL_ERRORS #1272). Each
//! repro's pdflatex text is in its header. apacite's full cites (#711) are rows of `error_boundary`.
use latexml::util::test::{assert_element, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

/// svjour3's `$\cdot$` separator, the `id`th math of the document.
macro_rules! cdot {
  ($id:literal) => {
    concat!(
      r#"<Math mode="inline" tex="\cdot" text="cdot" xml:id=""#,
      $id,
      r#""><XMath xml:id=""#,
      $id,
      r#".1"><XMTok name="cdot" role="MULOP">⋅</XMTok></XMath></Math>"#
    )
  };
}

/// One row: name, repro, the creators' names in order, the whole elements (tag, attribute or "" for the first of the
/// tag, element).
type Row = (
  &'static str,
  &'static str,
  &'static [&'static str],
  &'static [(&'static str, &'static str, &'static str)],
);

const ROWS: &[Row] = &[
  (
    "llncs `\\keywords` in the abstract",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/llncs_keywords_and_is_the_class_separator.tex"
    ),
    &["A", "B"],
    &[(
      "keywords",
      "",
      r#"<keywords name="Keywords: ">Alpha · Beta gamma · Delta</keywords>"#,
    )],
  ),
  (
    "llncs `\\keywords` before `\\author`: its `\\and` stays its own",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/llncs_keywords_and_before_the_authors.tex"
    ),
    &["A", "B"],
    &[(
      "keywords",
      "",
      r#"<keywords name="Keywords: ">One · Two</keywords>"#,
    )],
  ),
  (
    "svjour3 `\\keywords`, `\\subclass`, `\\PACS`, `\\CRclass`",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_keyword_lists_and_is_the_class_separator.tex"
    ),
    &["A", "B"],
    &[
      (
        "keywords",
        "",
        concat!(
          r#"<keywords name="Keywords: ">Alpha "#,
          cdot!("m1"),
          " Beta gamma ",
          cdot!("m2"),
          " Delta</keywords>"
        ),
      ),
      (
        "classification",
        r#"scheme="MSC""#,
        concat!(
          r#"<classification name="Mathematics Subject Classification (2000):" scheme="MSC">11A "#,
          cdot!("m3"),
          " 22B</classification>"
        ),
      ),
      (
        "classification",
        r#"scheme="pacs""#,
        concat!(
          r#"<classification name="PACS:" scheme="pacs">01.02 "#,
          cdot!("m4"),
          " 03.04</classification>"
        ),
      ),
      (
        "classification",
        r#"scheme="CR""#,
        concat!(
          r#"<classification name="CR Subject Classification:" scheme="CR">C.1 "#,
          cdot!("m5"),
          " D.2</classification>"
        ),
      ),
    ],
  ),
];

/// Each list keeps every item, each `\and` the class's separator between them; the authors still split at `\and`.
#[test]
fn class_keyword_lists_print_their_and() {
  for (name, tex, names, elements) in ROWS {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}:\n{log}"
    );
    let found: Vec<String> = creators(&xml).into_iter().map(|c| c[0].clone()).collect();
    assert_eq!(found, *names, "{name}:\n{xml}");
    for (tag, attr, element) in *elements {
      let attrs: &[&str] = if attr.is_empty() {
        &[]
      } else {
        std::slice::from_ref(attr)
      };
      assert_element(&xml, tag, attrs, element);
    }
  }
}
