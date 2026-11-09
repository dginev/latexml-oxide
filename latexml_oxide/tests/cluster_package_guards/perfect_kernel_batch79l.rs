//! Red/green guards for batch 79l (complete breakage, continued: user 2026-10-24): a front matter item that leaves a
//! group open closes it with itself, so the item's own `\label` binding does not outlive it; and the ACM template's
//! `\sharedaffiliation` is the line break it prints. Each guard converts its repro under `tools/perfect_kernel/repros/`,
//! asserts the diagnostics by count and the element whole.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The ACM SIG template's `\def\sharedaffiliation{\end{tabular}\begin{tabular}{c}}` in `\author` is the line break it
/// prints between the name and the shared affiliation (frontmatter/fallback.rs `tabular_juggles_as_line_breaks`):
/// no error, the affiliation kept, the labels with it. Red (int83a): 6 errors and a warning, the affiliation lost and
/// every later `\label` with it. Witnesses 1704.03168, 1301.5612, 1401.6981, 0712.1016. Repro
/// sectioning-frontmatter/author_shared_affiliation_line.
#[test]
fn a_shared_affiliation_is_the_authors_affiliation_line() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/author_shared_affiliation_line.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>A One</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Dept</contact></creator>",
  );
  assert!(
    xml.contains(r#"<section inlist="toc" labels="LABEL:sec:a" xml:id="S1">"#),
    "{xml}"
  );
}
