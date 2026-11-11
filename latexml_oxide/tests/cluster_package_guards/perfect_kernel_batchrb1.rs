//! Guards for batches RB1 and RB1c: REVTeX's `\maketitle` switches the author block off (revtex4-2.cls:2670-2684,
//! revtex4-1.cls:2649-2662, revtex4.cls:4709-4717), and ours switches off only a block the document repeats after it,
//! from its first `\author` on (revtex4_cls.rs `switch_off_author_block_after_maketitle`). Red (int159a): the repeat's
//! `\affiliation` on the last author and its names as body text (2401.15572, 1801.07635, 1812.00931, 2510.13035,
//! 2602.09975, 2307.07324, 2001.07542, 2111.12484). Red (int160b, RB1): a setter used only after `\maketitle` lost its
//! text (1309.4768, 1106.2389, 2304.09163) and a document's `\renewcommand{\author}` was refused (2412.14256). One
//! table: each row converts its repro once and asserts the exact Error and Warning counts, every creator's name and
//! contacts in order, the title and the body's text. A module of its own: no table covers what a class's `\maketitle`
//! leaves its setters doing.

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
  (class: $name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $name,
      ".tex"
    ))
  };
}

/// A row: its name, the document, each creator's name and contacts, the text after the abstract (else after the last
/// creator, else the title), and an element the XML holds (empty: none asserted).
type Case<'a> = (&'a str, String, &'a [&'a [&'a str]], &'a str, &'a str);

/// Rows: the repeated block under each class binding that runs the switch — revtex4-2 and revtex4 (revtex4_cls.rs, the
/// REVTeX 4.0 branch defining its own `\email`), revtex4-1 (revtex4_1_cls.rs) —, a second `\title` alone, kept as body
/// text, a document's own `\maketitle` (2509.25344), after which the switch yields and a `\date` is the document's date,
/// and its own `\frontmatter@maketitle` (2111.12484), which prints a second title, kept as text. RB1c's rows: an
/// `\affiliation`, `\email` or `\thanks` used only after `\maketitle` stays the live setter (1309.4768, 1106.2389,
/// 2304.09163), until a repeated block's `\author` (the lone affiliation then a block), and a `\renewcommand{\author}`
/// after it takes effect (2412.14256). Red (int159a): Bob Baker also held "Univ X, Town" and the body read "Text.
/// Supplement Ann Able Bob Baker More."; red (int160b): the RB1c rows' contacts, thanks text and names lost.
#[test]
fn revtex_maketitle_switches_the_author_block_off() {
  let block = repro!("revtex_repeated_author_block_after_maketitle_prints_nothing");
  let title = repro!("revtex_title_after_maketitle_prints_as_text");
  let lone_contacts = repro!(class: "revtex_lone_email_thanks_after_maketitle_are_kept");
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
  let both_x: &[&[&str]] = &[&["Ann Able", "affiliation=Univ X, Town"], &[
    "Bob Baker",
    "affiliation=Univ X, Town",
  ]];
  let bob_email: &[&[&str]] = &[&["Ann Able"], &["Bob Baker", "email=bob.baker@y.org"]];
  let cases: [Case; 13] = [
    (
      "revtex4-2",
      block.to_string(),
      two,
      "Text. Supplement More.",
      "",
    ),
    (
      "revtex4-1",
      block.replace("{revtex4-2}", "{revtex4-1}"),
      two,
      "Text. Supplement More.",
      "",
    ),
    (
      "revtex4",
      block.replace("{revtex4-2}", "{revtex4}"),
      two,
      "Text. Supplement More.",
      "",
    ),
    (
      "title",
      title.to_string(),
      one,
      "Text. Supplement More.",
      "",
    ),
    (
      "title, revtex4-1",
      title.replace("{revtex4-2}", "{revtex4-1}"),
      one,
      "Text. Supplement More.",
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
    (
      "lone affiliation, revtex4-1",
      repro!(class: "revtex_lone_affiliation_after_maketitle_is_kept").to_string(),
      both_x,
      "Text.",
      "",
    ),
    (
      "new affiliation",
      repro!(class: "revtex_new_affiliation_after_maketitle_is_kept").to_string(),
      &[&["Ann Able", "affiliation=Univ X, Town"], &[
        "Bob Baker",
        "affiliation=Univ Y, City",
        "affiliation=Univ Z, Village",
      ]],
      "Text.",
      "",
    ),
    (
      "lone email and thanks, revtex4",
      lone_contacts.to_string(),
      bob_email,
      "Funded by Z. Text.",
      "",
    ),
    (
      "lone email and thanks, revtex4-2",
      lone_contacts.replace("{revtex4}", "{revtex4-2}"),
      bob_email,
      "Funded by Z. Text.",
      "",
    ),
    (
      "lone affiliation, then a repeated block",
      repro!(class: "revtex_affiliation_after_maketitle_then_repeated_block").to_string(),
      both_x,
      "Text. Supplement More.",
      "",
    ),
    (
      "author renewed after maketitle",
      repro!(class: "revtex_author_renewed_after_maketitle").to_string(),
      &[],
      "Ann Able 1 , Bob Baker 2 1 Univ X Text.",
      r#"Ann Able<sup xml:id="p1.1.1"><text font="italic" xml:id="p1.1.1.1">1</text></sup>, Bob Baker<sup"#,
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
    let after = ["</abstract>", "</creator>", "</title>"]
      .iter()
      .find_map(|end| xml.rsplit_once(end))
      .expect("a title")
      .1;
    assert_eq!(text(after), body, "{name}: {xml}");
    assert!(xml.contains(holds), "{name}: {holds}: {xml}");
  }
}
