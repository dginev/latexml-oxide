//! Red/green guards for batch 68x: raw-class frontmatter rows from the int16 A/B. Item 1 — K11 for every class read
//! raw: `input_definitions` fires the title-page store reroutes at the end of a class load for the class a binding
//! read raw (apa6, apa, beilstein, lni, …), as for one the rawclasses lever reads, the binding's own setters staying its
//! own (68u round 3 called it from four bindings by hand; 2608.21389, 2102.04789).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch57::{RAW, assert_elements};

/// class-bindings/lni_subtitle_through_k11: lni.cls's `\subtitle` (a store only its refused `\maketitle` reads) is the
/// frontmatter's subtitle though the class is read by the lni binding, not the lever (pdflatex: "The Title" over "A
/// Subtitle"). Before 68u r3 a bold paragraph; 68u r3 by an explicit call in the binding, 68x by the one rule.
#[test]
fn a_class_a_binding_reads_raw_reroutes_its_stores() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/lni_subtitle_through_k11.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(&xml, "subtitle", &[], "<subtitle>A Subtitle</subtitle>");
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>Ann Able</personname><contact name="Email: " role="email">ann@example.org</contact><contact name="Affiliation: " role="affiliation">First University</contact></creator>"#,
  );
}
