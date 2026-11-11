//! Guards for batch RB1: REVTeX's `\maketitle` switches the author block's setters off once it has set the block
//! (revtex4-2.cls:2670-2684, revtex4-1.cls:2649-2662, revtex4.cls:4709-4717: `\affiliation`, `\author`, `\email`,
//! `\thanks` gobble their argument, `\and` is `\relax`) and keeps `\title` a silent setter, so a block a document
//! repeats after it (a supplement's) prints nothing (revtex4_cls.rs `switch_off_author_block_after_maketitle`). Red
//! (int159a): the repeat's `\affiliation` on the last author and its names and title as body text (2401.15572,
//! 1801.07635, 1812.00931, 2510.13035, 2602.09975, 2307.07324, 2001.07542, 2111.12484). One table: each row converts
//! its repro once and asserts the exact Error and Warning counts, every creator's name and contacts in order, the title
//! and the body's text. A module of its own: no table covers what a class's `\maketitle` leaves its setters doing.

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::{creators, text},
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A row: its name, the document, each creator's name and contacts, the text after the abstract, and an element the
/// XML holds (empty: none asserted).
type Case<'a> = (&'a str, String, &'a [&'a [&'a str]], &'a str, &'a str);

/// Rows: the repeated block under each class binding that runs the switch — revtex4-2 and revtex4 (revtex4_cls.rs, the
/// REVTeX 4.0 branch defining its own `\email`), revtex4-1 (revtex4_1_cls.rs) —, a second `\title` alone (the silent
/// setter) under revtex4-2 and revtex4-1, a document's own `\maketitle` (2509.25344), after which the switch
/// yields and a `\date` is the document's date, and its own `\frontmatter@maketitle` (2111.12484), which prints a
/// second title, kept as text. Red (int159a): Bob Baker also held "Univ X, Town", the body read
/// "Text. Supplement Ann Able Bob Baker More." and "Text. Supplement More."; RB1's first cut lost the own-`\maketitle`
/// row's date and the own-`\frontmatter@maketitle` row's title.
#[test]
fn revtex_maketitle_switches_the_author_block_off() {
  let block = repro!("revtex_repeated_author_block_after_maketitle_prints_nothing");
  let title = repro!("revtex_title_after_maketitle_stores_silently");
  let two: &[&[&str]] = &[
    &[
      "Ann Able",
      "email=ann.able@x.org",
      "affiliation=Univ X, Town",
    ],
    &[
      "Bob Baker",
      "email=bob.baker@y.org",
      "affiliation=Univ Y, City",
    ],
  ];
  let one: &[&[&str]] = &[&["Ann Able", "affiliation=Univ X, Town"]];
  let cases: [Case; 7] = [
    ("revtex4-2", block.to_string(), two, "Text. More.", ""),
    (
      "revtex4-1",
      block.replace("{revtex4-2}", "{revtex4-1}"),
      two,
      "Text. More.",
      "",
    ),
    (
      "revtex4",
      block.replace("{revtex4-2}", "{revtex4}"),
      two,
      "Text. More.",
      "",
    ),
    ("title", title.to_string(), one, "Text. More.", ""),
    (
      "title, revtex4-1",
      title.replace("{revtex4-2}", "{revtex4-1}"),
      one,
      "Text. More.",
      "",
    ),
    (
      "own maketitle",
      repro!("revtex_own_maketitle_keeps_the_setters").to_string(),
      one,
      "Text.",
      r#"<date role="creation">May 5</date>"#,
    ),
    (
      "own frontmatter@maketitle",
      repro!("revtex_own_frontmatter_maketitle_prints_the_title_again").to_string(),
      one,
      "Text. Supplement More.",
      "",
    ),
  ];
  for (name, tex, expected, body, holds) in cases {
    let (log, xml) = latexml::util::test::convert_files_with(&tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|row| row.iter().map(ToString::to_string).collect())
      .collect();
    assert_eq!(creators(&xml), expected, "{name}: {xml}");
    assert_eq!(xml.matches("<title>").count(), 1, "{name}: {xml}");
    assert!(xml.contains("<title>T</title>"), "{name}: {xml}");
    let after = xml.split("</abstract>").nth(1).expect("abstract");
    assert_eq!(text(after), body, "{name}: {xml}");
    assert!(xml.contains(holds), "{name}: {holds}: {xml}");
  }
}
