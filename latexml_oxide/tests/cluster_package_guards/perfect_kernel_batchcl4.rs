//! Guards for CL4 (`agent_reports/2026-11-08_brief_CL4.md`; w56's report, SYNC leftovers (33)): the frontmatter observer
//! reads a pgf/TikZ picture as it prints, its text nodes only (`util_text::ink_text_printed`, `picture_text`), never its
//! drawing: a TikZ ORCID logo's SVG path data was a link's text, written into the address line at `=2`
//! (2409.13798, jacow).

use latexml::util::test::{error_count, warning_count};

use super::perfect_kernel_batchz3::in_a_directory;

/// `R. Apsimon\orcidB{}` with `\orcidB` an `\href` around a `tikzpicture` (a circle and its "ID" node): at `=2` no path
/// data (`M 0 0 M 6.3 0 C …`) reaches the frontmatter or the log, the content check finds no missing or extra word, and
/// the logo's printed "ID" is kept beside the name; off is unchanged. One guard: one reader of an event's ink.
#[test]
fn a_tikz_icons_path_data_is_no_word() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/emit_orcid_tikz_icon_no_path_text.tex"
  );
  let warnings = 2;
  let (log, xml) = in_a_directory(tex, &[], &[]);
  assert_eq!(
    (error_count(&log), log.matches("\nFatal:").count()),
    (0, 0),
    "{log}"
  );
  assert_eq!(warning_count(&log), warnings, "{log}");
  assert!(!log.contains("Frontmatter content check"), "{log}");
  let frontmatter: String = xml
    .split("<creator")
    .skip(1)
    .map(|c| c.split("</creator>").next().unwrap_or_default())
    .collect();
  for path in ["M 0 0", "6.3 0 C", "1.00.00.01.0"] {
    assert!(!log.contains(path), "{log}");
    assert!(!frontmatter.contains(path), "{frontmatter}");
  }
  assert!(
    frontmatter.contains("R. Apsimon ID, S. Setiniyaz, Lancaster University, UK"),
    "{frontmatter}"
  );
}
