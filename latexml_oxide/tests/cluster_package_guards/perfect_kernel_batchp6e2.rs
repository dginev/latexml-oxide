//! P6e2 (CL169's P6e regression): an e-mail printed in a block several names share — after a run of two or more names
//! with no entry between, or in a document's note after them — goes to the name of the run it spells; one after a single
//! name is its own call, kept whatever it spells (`place::shared_block_email_labels`, the queue's respell for what the
//! reading does not place).

use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch61::creators_of,
};

/// Each row: a repro, its creators whole, its (Error, Warning) counts. Red on int169a (the shared block's two addresses
/// both on Bob Baker: 2310.06444, 2601.17008, 2504.00797; past a name's `\authornote`, 1907.07581); the own-call row pins the better side P6e gave
/// (2208.05647, 2306.14314, 2609.11148; the address two own calls give on both, 2205.01906, 2403.05668), which int168a's
/// DOM respell got wrong. Distinct: the run of names before the entry, two or more, against one.
#[test]
fn a_shared_block_email_goes_to_the_name_it_spells() {
  let rows: Vec<(&str, Vec<String>, (usize, usize))> = vec![
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/acmart_shared_block_email_goes_to_the_name_it_spells.tex"
      ),
      vec![
        "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text class=\"ltx_affiliation_institution\" xml:id=\"id1\">X Univ</text>, <text class=\"ltx_affiliation_city\" xml:id=\"id2\">C</text>, <text class=\"ltx_affiliation_country\" xml:id=\"id3\">D</text></contact><contact name=\"email: \" role=\"email\">able@x.org</contact></creator>".to_string(),
        "<creator before=\" and \" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text class=\"ltx_affiliation_institution\" xml:id=\"id4\">X Univ</text>, <text class=\"ltx_affiliation_city\" xml:id=\"id5\">C</text>, <text class=\"ltx_affiliation_country\" xml:id=\"id6\">D</text></contact><contact name=\"email: \" role=\"email\">baker@x.org</contact></creator>".to_string(),
      ],
      (0, 0),
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/acmart_own_email_call_stays_whatever_it_spells.tex"
      ),
      vec![
        "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text class=\"ltx_affiliation_institution\" xml:id=\"id1\">X Univ</text>, <text class=\"ltx_affiliation_city\" xml:id=\"id2\">C</text>, <text class=\"ltx_affiliation_country\" xml:id=\"id3\">D</text></contact><contact name=\"email: \" role=\"email\">bbaker@x.org</contact><contact name=\"email: \" role=\"email\">lab@x.org</contact></creator>".to_string(),
        "<creator before=\" and \" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text class=\"ltx_affiliation_institution\" xml:id=\"id4\">Y Univ</text>, <text class=\"ltx_affiliation_city\" xml:id=\"id5\">C</text>, <text class=\"ltx_affiliation_country\" xml:id=\"id6\">D</text></contact><contact name=\"email: \" role=\"email\">bob@y.org</contact><contact name=\"email: \" role=\"email\">lab@x.org</contact></creator>".to_string(),
      ],
      (0, 1),
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/acmart_shared_block_email_past_an_authornote_goes_to_the_name_it_spells.tex"
      ),
      vec![
        "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Note:\u{a0}\" role=\"note\">Both authors contributed equally.</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text class=\"ltx_affiliation_institution\" xml:id=\"id1\">X Univ</text>, <text class=\"ltx_affiliation_city\" xml:id=\"id2\">C</text>, <text class=\"ltx_affiliation_country\" xml:id=\"id3\">D</text></contact><contact name=\"email: \" role=\"email\">able@x.org</contact></creator>".to_string(),
        "<creator before=\" and \" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text class=\"ltx_affiliation_institution\" xml:id=\"id4\">X Univ</text>, <text class=\"ltx_affiliation_city\" xml:id=\"id5\">C</text>, <text class=\"ltx_affiliation_country\" xml:id=\"id6\">D</text></contact><contact name=\"email: \" role=\"email\">baker@x.org</contact></creator>".to_string(),
      ],
      (0, 0),
    ),
  ];
  let mut differ = Vec::new();
  for (row, (tex, expected, counts)) in rows.iter().enumerate() {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    let found = (creators_of(&xml), (error_count(&log), warning_count(&log)));
    if found != (expected.clone(), *counts) {
      differ.push(format!("row {row}: {found:#?}\n{log}"));
    }
  }
  assert!(differ.is_empty(), "{}", differ.join("\n\n"));
}
