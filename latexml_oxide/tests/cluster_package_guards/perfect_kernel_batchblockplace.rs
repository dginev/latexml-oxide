//! Guards for BLOCKPLACE: an author-block contact note no creator holds sits right after the last creator, before the
//! abstract, as `\maketitle` prints it (`insert::place_block_notes`, OXIDIZED_DESIGN_DIVERGENCES #1074; DG
//! 2026-11-17); with no creator it keeps Perl's place after the abstract. Each asserts the exact Error and Warning
//! counts, the frontmatter's order at the root and the note whole.

use latexml::util::test::{error_count, warning_count};

use super::perfect_kernel_batchz4::run;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The document's children before its first section, each its name, a note's with its role.
fn frontmatter_order(xml: &str) -> Vec<String> {
  xml
    .lines()
    .take_while(|line| !line.starts_with("  <section"))
    .filter_map(|line| line.strip_prefix("  <"))
    .filter(|rest| rest.starts_with(|c: char| c.is_ascii_alphabetic()))
    .map(|rest| {
      let name: String = rest
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect();
      match rest.split_once("role=\"") {
        Some((_, role)) if name == "note" => format!("note[{}]", &role[..role.find('"').unwrap()]),
        _ => name,
      }
    })
    .collect()
}

/// jfm's shared, unmarked `\affiliation` under two names is the block's note (1711.01065; 2603.01467's Interspeech
/// `\email`, mdpi 1910.13722, ieeeaccess 2006.03169, oup-authoring-template 2512.03760): placed after the creators,
/// before the abstract, the PDF's order. Red (208b29241bb): after the abstract. Distinct: the move, and the no-creator
/// path that keeps Perl's order.
#[test]
fn a_block_contact_note_sits_under_the_names() {
  let (log, xml) = run(repro!("block_contact_note_before_abstract"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    frontmatter_order(&xml),
    [
      "title",
      "creator",
      "creator",
      "note[affiliation]",
      "abstract"
    ],
    "{xml}"
  );
  assert!(
    xml.contains(
      "  </creator>\n  <note role=\"affiliation\" xml:id=\"id1\">Univ X, Town</note>\n  <abstract "
    ),
    "{xml}"
  );

  let (log, xml) = run(repro!("block_contact_note_without_creators"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    frontmatter_order(&xml),
    ["title", "abstract", "note[affiliation]"],
    "{xml}"
  );
  assert!(
    xml.contains("  <note role=\"affiliation\" xml:id=\"id1\">Univ X, Town</note>\n"),
    "{xml}"
  );
}
