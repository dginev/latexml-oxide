//! Guards for batch P21e (OD #993 P2-1c, scoped): the events' walker reads a paragraph's list as no group only at the
//! block's own level — the `{CJK*}` repack of 2608.21138 (`perfect_kernel_batchp21c`). A paragraph inside a box's body
//! (a minipage, a `\parbox`) stays a group, as before P21c: read as no group, the mark's group became the box, whose span
//! reaches the box's unshown arguments and whose run took a contact's markup (`events/mod.rs` `paragraph_is_no_group`).
//! The repros compile clean with pdflatex (TL 2025); the guards assert every creator's name and contacts, in order, the
//! contact's whole element, and the exact Error and Warning counts.

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

fn rows(rows: &[&[&str]]) -> Vec<Vec<String>> {
  rows
    .iter()
    .map(|row| row.iter().map(ToString::to_string).collect())
    .collect()
}

/// A mark in a box with an unshown position and width argument (`[t]{0.47\linewidth}`) keeps the paragraph as its group:
/// the contact holds the box's text alone (2608.17538: "t]0.47" and `\centering` read into each `\dedicatory` contact,
/// two "Missing number" Warnings). Two rows, two code paths of the container walk: an environment box (`{minipage}`,
/// its arguments settings) and a command box (`\parbox`, its shown arguments filtered), whose contacts were written as
/// the empty box.
#[test]
fn a_paragraph_in_a_box_stays_a_group() {
  let cases = [
    (
      "minipage",
      repro!("amsart_dedicatory_box_paragraph_stays_a_group"),
      [
        r#"<contact role="dedicatory"><sup xml:id="id2">1</sup>Univ X<break/>City Y</contact>"#,
        r#"<contact role="dedicatory"><sup xml:id="id4">2</sup>Univ Z</contact>"#,
      ],
    ),
    (
      "parbox",
      repro!("amsart_dedicatory_parbox_paragraph_stays_a_group"),
      [
        r#"<contact role="dedicatory"><sup class="ltx_centering" xml:id="id2">1</sup>Univ X<break class="ltx_centering"/>City Y</contact>"#,
        r#"<contact role="dedicatory"><sup class="ltx_centering" xml:id="id4">2</sup>Univ Z</contact>"#,
      ],
    ),
  ];
  for (name, tex, contacts) in cases {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert_eq!(
      creators(&xml),
      rows(&[&["Ann Able 1", "dedicatory=1 Univ X City Y"], &[
        "Bob Baker 2",
        "dedicatory=2 Univ Z"
      ],]),
      "{name}: {xml}"
    );
    for contact in contacts {
      assert_eq!(xml.matches(contact).count(), 1, "{name}: {contact}: {xml}");
    }
  }
}

/// A contact in a box beside a braced mark keeps its own markup: the paragraph a `\par` or the box packs stays the mark's
/// group, so the contact is written from its source run, not as its address alone. Two rows, two contact writers: an
/// e-mail line's `\texttt` (2402.02104, authblk `{minipage}`) and a `\thanks`'s `\href{mailto:…}` (2601.03873, mnras
/// `\parbox`; its owner Ann Able is int157a's, the address printed in Cy Cole's footnote — pre-existing, outside this
/// batch). That row pins a KNOWN-WRONG owner (SILENT_LOSS_INVENTORY "Bundle int158a — the residual"): a change moving
/// the address to Cy Cole is a fix, not a regression; update the row then.
#[test]
fn a_paragraph_in_a_box_keeps_a_contacts_markup() {
  let cases = [
    (
      "minipage texttt",
      repro!("authblk_minipage_paragraph_keeps_a_typewriter_email"),
      (0, 0),
      rows(&[&["Ann Able 1", "email=a@x.org", "affiliation=Univ X"]]),
      "<contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">a@x.org</text></contact>",
    ),
    (
      "parbox href",
      repro!("mnras_parbox_paragraph_keeps_a_mailto_link"),
      (0, 2),
      rows(&[
        &["Team X: Ann Able 1", "email=team@x.org"],
        // (the `\footnotemark`'s note, its tags' text, in the name: int157a's)
        &["Bob Baker 1 footnote 1 1 footnote 1 2"],
        &["Cy Cole 3"],
      ]),
      "<contact name=\"Email:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" href=\"mailto:team@x.org\">team@x.org</ref></contact>",
    ),
  ];
  for (name, tex, counts, expected, contact) in cases {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      counts,
      "{name}: {log}"
    );
    assert_eq!(creators(&xml), expected, "{name}: {xml}");
    assert_eq!(xml.matches(contact).count(), 1, "{name}: {xml}");
  }
}
