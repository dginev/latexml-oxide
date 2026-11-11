//! Guards for batch P21c (Option C step P2-1c, OD #993): aa's `\author` hands its whole name list to the reading in one
//! `\lx@add@author@split` call, where it split it by spelling first (`\lx@splitting`). A `{CJK*}` environment in a later
//! name leaves horizontal mode, whose `\par` packs the block's line so far into a paragraph list; the events' walker
//! reads that list as no group (`events/mod.rs` `paragraph_is_no_group`), so an earlier name's `\inst` stays a mark at the
//! block's level and the contact after it keeps its own call. The repros compile clean with pdflatex (TL 2025,
//! 2608.21138's aa.cls); the guard asserts every creator's name and contacts, in order, the contact's whole element, and
//! the exact Error and Warning counts.

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/author_one_block/",
      $name,
      ".tex"
    ))
  };
}

/// A contact after a name's `\inst`, in the block a later name's `{CJK*}` closes a paragraph of, keeps its own call
/// (2608.21138: `\corrauth` kept `name="Corresponding author: "`; read as a group's mark, the paragraph made it a plain
/// contact written by its text, with aa's split removed). Three rows: `\corrauth` (the emitter's call path), aa_support's
/// `\email` (its e-mail branch, the address's typewriter font from its events, which the text path lost) and the other
/// order, the `{CJK*}` name before the `\corrauth` author (a pin: the paragraph never holds the contact).
#[test]
fn a_contact_keeps_its_call_beside_a_cjk_name() {
  let rows = |rows: &[&[&str]]| -> Vec<Vec<String>> {
    rows
      .iter()
      .map(|row| row.iter().map(ToString::to_string).collect())
      .collect()
  };
  let cases = [
    (
      "corrauth before",
      repro!("aa_corrauth_beside_a_cjk_name_in_one_block"),
      rows(&[
        &[
          "Ann M. Able",
          "corresponding=ann@x.edu",
          "affiliation=University of X",
        ],
        &[
          "Bob Baker(贾彬)",
          "affiliation=University of X",
          "affiliation=University of Y",
        ],
        &["Cid Cole", "affiliation=University of Y"],
      ]),
      r#"<contact name="Corresponding author: " role="corresponding">ann@x.edu</contact>"#,
    ),
    (
      "email before",
      repro!("aa_email_beside_a_cjk_name_in_one_block"),
      rows(&[
        &[
          "Ann M. Able",
          "email=ann@x.edu",
          "affiliation=University of X",
        ],
        &[
          "Bob Baker(贾彬)",
          "affiliation=University of X",
          "affiliation=University of Y",
        ],
        &["Cid Cole", "affiliation=University of Y"],
      ]),
      r#"<contact name="Email: " role="email"><text font="typewriter" xml:id="id1">ann@x.edu</text></contact>"#,
    ),
    (
      "corrauth after",
      repro!("aa_corrauth_after_a_cjk_name_in_one_block"),
      rows(&[
        &["Ann M. Able", "affiliation=University of X"],
        &[
          "Bob Baker(贾彬)",
          "affiliation=University of X",
          "affiliation=University of Y",
        ],
        &[
          "Cid Cole",
          "corresponding=cid@y.edu",
          "affiliation=University of Y",
        ],
      ]),
      r#"<contact name="Corresponding author: " role="corresponding">cid@y.edu</contact>"#,
    ),
  ];
  for (name, tex, expected, contact) in cases {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert_eq!(creators(&xml), expected, "{name}: {xml}");
    assert_eq!(xml.matches(contact).count(), 1, "{name}: {xml}");
  }
}
