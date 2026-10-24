//! Red/green guards for batch 79m (complete breakage, continued: user 2026-10-24): REVTeX's `\email` and `\homepage`
//! read the address with `%` other, as the class does, so a `%` in it is the address and not a comment that swallows
//! the rest of the document. Each guard converts its repro under `tools/perfect_kernel/repros/`, asserts the
//! diagnostics by count and the element whole.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `\homepage{http://www.nhn.ou.edu/%7Emilton}` under revtex4 is the address (revtex4.cls:4476-4490 `\begingroup
/// \sanitize@url\@homepage{#1}`, `%` other; revtex4_support_sty.rs `\homepage`), and so is `\email{one%two@a.edu}`;
/// the section and the equation after them keep their labels. Red (int85a): the `%` a comment, the address argument
/// ran to the end of the document, 10 errors, the equation's label lost. Witness 0712.3811 (76 of 84 references
/// dangling). Repro sectioning-frontmatter/revtex_homepage_percent.
#[test]
fn a_percent_in_a_revtex_address_is_the_address() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/revtex_homepage_percent.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "contact",
    &["role=\"url\""],
    "<contact name=\"URL:\u{a0}\" role=\"url\"><text font=\"typewriter\" xml:id=\"id2\">http://www.nhn.ou.edu/%7Emilton</text></contact>",
  );
  assert_element(
    &xml,
    "contact",
    &["role=\"email\""],
    "<contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">one%two@a.edu</text></contact>",
  );
  assert!(
    xml.contains(r#"<section inlist="toc" labels="LABEL:sec:a" xml:id="S1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<equation labels="LABEL:eq:b" xml:id="S1.E1">"#),
    "{xml}"
  );
}

/// amsart's `\subjclass` is not `\long` (amsart.cls:555 `\newcommand*\subjclass`; ams_support_sty.rs `short => true`,
/// with `\title`, `\keywords`, `\dedicatory`, `\translator`): a `\subjclass{…: 65N12, 74G65` whose `}` never comes
/// ends at the blank line, "Paragraph ended before \subjclass was complete", and the section after it keeps its place
/// and its label. Red (int85a): the argument ran to the end of the document, 6 errors, the section inside the
/// classification and its reference dangling. Witness 2212.11488 (81 of 153 references dangling). Repro
/// sectioning-frontmatter/ams_subjclass_unclosed.
#[test]
fn an_unclosed_subjclass_ends_at_the_paragraph() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/ams_subjclass_unclosed.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.contains("Paragraph ended before \\subjclass was complete"),
    "{log}"
  );
  assert!(
    xml.contains(r#"<section inlist="toc" labels="LABEL:sec:a" xml:id="S1">"#),
    "{xml}"
  );
  assert!(!xml.contains("<classification"), "{xml}");
  assert!(xml.contains(r#"<ref labelref="LABEL:sec:a""#), "{xml}");
}
