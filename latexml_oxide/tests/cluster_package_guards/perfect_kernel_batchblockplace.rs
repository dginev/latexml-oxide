//! Guards for BLOCKPLACE: a note its binding declares printed under the names (`\lx@names@note`, `_zone=names`) sits
//! right after the last creator, before the abstract, as the class prints it (`insert::place_block_notes`,
//! OXIDIZED_DESIGN_DIVERGENCES #1074; DG 2026-11-17); an undeclared note, or one with no creator, keeps Perl's place
//! after the abstract. Each asserts the exact Error and Warning counts, the frontmatter's order at the root and the note
//! whole.

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

/// jfm's shared, unmarked `\affiliation` under two names, declared under the names (1711.01065; the declared siblings
/// mdpi 1910.13722, ieeeaccess 2006.03169, oup-authoring-template 2512.03760, sigma 0803.4168, autart 1402.5644):
/// placed after the creators, before the abstract, the PDF's order. Red (208b29241bb): after the abstract. Distinct:
/// the move, and the no-creator path that keeps Perl's order.
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

/// tac's `\address`, which tac.cls:611-614 prints `\AtEndDocument`, is not declared: it keeps Perl's place after the
/// abstract (1504.03886; sagej's footnoted `\affiliation`, 1910.13936, the same path). Red (dbaea4ca220, every
/// contact-role note moved): before the abstract. Distinct: the opt-in, an undeclared contact-role note that stays.
#[test]
fn an_undeclared_note_keeps_perls_place() {
  let (log, xml) = run(repro!("tac_block_note_keeps_its_place"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    frontmatter_order(&xml),
    ["title", "creator", "abstract", "note[address]"],
    "{xml}"
  );
  assert!(
    xml.contains("  <note role=\"address\" xml:id=\"id1\">Univ X, Town</note>\n"),
    "{xml}"
  );
}
