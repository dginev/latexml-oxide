//! Guards of behaviour the interpreter's hot paths must keep (`docs/performance/PERFORMANCE.md`, "the pk/P1 levers").
//!
//! An expansion that a printed field goes through comes back to `invoke_expansion` as a list, which `inherit_field_prov`
//! stamps with the field's provenance, and the raw print is then told from a body paragraph. `\expandafter` once expanded
//! straight onto the input for its speed: the title stayed in the body as well, with no Error or Warning.
use super::{perfect_kernel_batch46::error_count, perfect_kernel_batch89a::cli_conversion};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

#[test]
fn a_title_printed_through_expandafter_is_lifted() {
  let (log, xml) = cli_conversion(repro!("title_printed_through_expandafter"), false);
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(
    xml.contains(r#"<title font="italic"><emph xml:id="id1">Hello World</emph></title>"#),
    "{xml}"
  );
  assert_eq!(xml.matches("Hello World").count(), 1, "{xml}");
}

#[test]
fn an_author_printed_through_expandafter_is_lifted() {
  let (log, xml) = cli_conversion(repro!("author_printed_through_expandafter"), false);
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(
    xml.contains("<personname>Hello World</personname>"),
    "{xml}"
  );
  // Title and author once each; the print is what would double it, in the one paragraph.
  assert_eq!(xml.matches("Hello World").count(), 2, "{xml}");
  assert!(xml.contains(r#"<p xml:id="p1.1">Body text.</p>"#), "{xml}");
}
