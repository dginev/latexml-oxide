//! Guards for BLOCKPLACE: a note its binding declares printed under the names (`\lx@names@note`, `_zone=names`) sits
//! right after the last creator, before the abstract, as the class prints it (`insert::place_block_notes`,
//! OXIDIZED_DESIGN_DIVERGENCES #1074; DG 2026-11-17); an undeclared note, or one with no creator, keeps Perl's place
//! after the abstract. Each asserts the exact Error and Warning counts, the frontmatter's order at the root and the note
//! whole.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch64::post_with, perfect_kernel_batchz4::run};

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

/// The contrib classes' declared notes (BLOCKDECL): the repro, the root's order without its creators, the creators, the
/// run from the last creator to the abstract, and the page's run from the names to the abstract where a moved note's
/// role is one `f:document-note-shown` gained (its text a line, no lone `†`).
type ClassRow = (
  &'static str,
  &'static [&'static str],
  usize,
  &'static str,
  Option<&'static str>,
);
const CLASS_ROWS: &[ClassRow] = &[
  // bmvc2k's `\addinstitution` (1507.06535, 1806.03891, 1908.01683)
  (
    repro!("bmvc2k_institution_note_under_the_names"),
    &["title", "toctitle", "note[affiliation]", "abstract"],
    1,
    "  </creator>\n  <note role=\"affiliation\" xml:id=\"id2\">Inst X<break/>City Y</note>\n  <abstract ",
    None,
  ),
  // bytedance_seed's `\affiliation` and `\contribution`, `\correspondence` after the abstract (2506.15211, 2511.06346)
  (
    repro!("bytedance_affiliation_and_contribution_under_the_names"),
    &[
      "title",
      "note[affiliation]",
      "note[contribution]",
      "abstract",
      "note[correspondence]",
    ],
    1,
    "  </creator>\n  <note role=\"affiliation\" xml:id=\"id1\">Inst X</note>\n  <note role=\"contribution\" \
     xml:id=\"id2\">Work done at Q</note>\n  <abstract ",
    Some(HTML_BYTEDANCE),
  ),
  // wlpeerj's `\affil` and `\corrauthor`'s e-mail (2007.11779), in either call order (the e-mail declared at
  // `\maketitle`); two creators for one author, `wlpeerj_corrauthor_second_creator_red`'s defect (pinned to move with it)
  (
    repro!("wlpeerj_affil_and_email_under_the_names"),
    &["title", "note[affiliation]", "note[email]", "abstract"],
    2,
    "  </creator>\n  <note role=\"affiliation\" xml:id=\"id1\">Univ X, Town</note>\n  <note role=\"email\" \
     xml:id=\"id2\">ann@x.edu</note>\n  <abstract ",
    None,
  ),
  (
    repro!("wlpeerj_corrauthor_before_affil_email_after"),
    &["title", "note[affiliation]", "note[email]", "abstract"],
    2,
    "  </creator>\n  <note role=\"affiliation\" xml:id=\"id1\">Univ X, Town</note>\n  <note role=\"email\" \
     xml:id=\"id2\">ann@x.edu</note>\n  <abstract ",
    None,
  ),
  // sn-jnl's `\presentaddress`, under the creator's `\equalcont` note (2412.01641's sn-jnl.cls:754-756)
  (
    repro!("snjnl_present_address_under_the_names"),
    &["title", "note[present-address]", "abstract"],
    1,
    "  </creator>\n  <note role=\"present-address\" xml:id=\"id2\">Lab Q, Town</note>\n  <abstract ",
    Some(HTML_SNJNL),
  ),
];

const HTML_BYTEDANCE: &str = "<div class=\"ltx_authors\">\n<span class=\"ltx_creator ltx_role_author\">\n<span \
                              class=\"ltx_personname\">Ann Able\n</span></span></div>\n<div id=\"id1\" class=\"ltx_note \
                              ltx_role_affiliation ltx_note_unmarked\">Inst X</div><div id=\"id2\" class=\"ltx_note \
                              ltx_role_contribution ltx_note_unmarked\">Work done at Q</div>\n";
const HTML_SNJNL: &str = "<div class=\"ltx_authors\">\n<span class=\"ltx_creator ltx_role_author\"><span \
                          class=\"ltx_annotated_personname\">\n<span class=\"ltx_personname\">Ann Able</span><span id=\"id1\" \
                          class=\"ltx_note ltx_role_equal-contributors\"><sup class=\"ltx_note_mark\">†</sup><span \
                          class=\"ltx_note_outer\"><span class=\"ltx_note_content\"><sup \
                          class=\"ltx_note_mark\">†</sup><span class=\"ltx_note_type\">equal-contributors: \
                          </span>These authors contributed equally to this work.</span></span></span></span><span \
                          class=\"ltx_author_notes\"><span class=\"ltx_author_notes_content\">\n<span \
                          class=\"ltx_contact ltx_role_affiliation\"><span class=\"ltx_contact_name\">Affiliation:\u{a0}\
                          </span>Univ A\n</span></span></span></span></div>\n<div id=\"id2\" class=\"ltx_note \
                          ltx_role_present-address ltx_note_unmarked\">Lab Q, Town</div>\n";

/// jfm's shared, unmarked `\affiliation` under two names, declared under the names (1711.01065; the declared siblings
/// mdpi 1910.13722, ieeeaccess 2006.03169, oup-authoring-template 2512.03760, sigma 0803.4168, autart 1402.5644):
/// placed after the creators, before the abstract, the PDF's order. Red (208b29241bb): after the abstract. Distinct:
/// the move, and the no-creator path that keeps Perl's order. The rows of [`CLASS_ROWS`], red on INT081RC6 (after the
/// abstract): the contrib classes' declarations, bytedance's two lists in order with the undeclared note after; the
/// wlpeerj e-mail given before the `\affil` and sn-jnl's present address, and the pages' contribution and present-address
/// lines, red on eca47cbf6cb (the e-mail first; after the abstract; a lone `†`, its text in the hover).
#[test]
fn a_block_contact_note_sits_under_the_names() {
  for &(tex, order, creators, block, html) in CLASS_ROWS {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let found = frontmatter_order(&xml);
    let found_creators = found.iter().filter(|name| *name == "creator").count();
    let others: Vec<&str> = found
      .iter()
      .map(String::as_str)
      .filter(|name| *name != "creator")
      .collect();
    assert_eq!(others, order, "{xml}");
    assert_eq!(found_creators, creators, "{xml}");
    assert!(xml.contains(block), "{xml}");
    // (the block's `</creator>` is the last creator's)
    assert!(
      !xml[xml.find(block).unwrap() + 1..].contains("<creator"),
      "{xml}"
    );
    if let Some(expected) = html {
      let page = post_with(&xml, "resources/XSLT/LaTeXML-html5.xsl");
      let names = page.find("<div class=\"ltx_authors\">").expect("the names");
      let to_abstract = &page[names
        ..page[names..]
          .find("<div id=\"abstract1\"")
          .map(|i| names + i)
          .expect("the abstract")];
      assert_eq!(to_abstract, expected, "{page}");
    }
  }

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
