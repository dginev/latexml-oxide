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

/// An author that leaves a tabular open (`\end{tabular}\begin{tabular}{c}` written in `\author`, the class's title-page
/// juggle) closes it with the item (frontmatter queue_digest.rs `digest_in_own_group`), and the section and the equation
/// keep their labels. Red (int83a): the text mode could not end, the item's group was never popped, and its `\label`
/// (`\lx@set@frontmatter@label`) took every later `\label`: no `labels` anywhere, every reference dangling. The item's
/// own errors (the unbalanced `\end{tabular}`) remain, 4 of them. Witnesses 1704.03168, 1301.5612, 1401.6981,
/// 0712.1016 (sig-alternate). Repro sectioning-frontmatter/author_group_left_open_keeps_labels.
#[test]
fn an_author_left_open_keeps_the_documents_labels() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/author_group_left_open_keeps_labels.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (4, 0), "{log}");
  assert!(
    xml.contains(r#"<section inlist="toc" labels="LABEL:sec:a" xml:id="S1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<equation labels="LABEL:eq:b" xml:id="S1.E1">"#),
    "{xml}"
  );
}

/// The ACM SIG template's `\def\sharedaffiliation{\end{tabular}\begin{tabular}{c}}` in `\author` is the line break it
/// prints between the name and the shared affiliation (author_block separators.rs `tabular_juggles_as_line_breaks`):
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
