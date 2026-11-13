//! Guards for MB1, the one label rule of the Kind step's lines (OXIDIZED_DESIGN_DIVERGENCES #992; review revZ18o N1,
//! N2): an entry's line led by a label of its own — an e-mail label's (`email_line_label`) or a mailbox link line's
//! text before its link (`mailbox_link_line`) — is named after the class's name of the entry where both print
//! (`entry_line_label`); a link line whose label is no e-mail label is the link line's, its link kept.
//!
//! Each converts its repros and asserts the exact Error and Warning counts, every creator's name and contacts in order,
//! and the e-mail contact whole.

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batchz4::run,
  perfect_kernel_batchz18o::{assert_creators, email_contacts},
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

/// An entry the class names — aa's `\mail` ("Correspondence to"), REVTeX's `\altaffiliation[Authors for
/// Correspondence: ]` — holding a line led by a label of its own: the e-mail is named by the class's name, then the
/// line's label, as both print, whichever reader reads the line (a mailbox link line, `Contact: \href{mailto:…}{here}`;
/// an e-mail label's line, `e-mail: zz@x.org`, 0805.1001). Red (int156a): each link line's e-mail named "Contact:"
/// alone; the address line green there, pinned as the rule's other reader. Red (MB1): aa's "Correspondence to  …",
/// a double space and no colon — the join added a space after the unexpanded `\mailname`, which held its own; aa.cls
/// prints "Correspondence to: …" (MB2, review revMB1 N1, P2).
#[test]
fn labelled_line_is_named_after_the_class() {
  let link = "<ref class=\"ltx_href\" href=\"mailto:zz@x.org\">here</ref>";
  for (tex, creators, contact) in [
    (
      repro!("aa_mail_labelled_mailbox_link_names_after_the_class"),
      &[&["A. Smith", "affiliation=Univ. X, Y", "email=here"][..]][..],
      format!("<contact name=\"Correspondence to: Contact: \" role=\"email\">{link}</contact>"),
    ),
    (
      repro!("revtex_altaffiliation_labelled_mailbox_link_names_after_the_class"),
      &[
        &["Ann Able", "affiliation=Univ A", "email=here"][..],
        &["Bob Baker", "affiliation=Univ A"][..],
      ][..],
      format!(
        "<contact name=\"Authors for Correspondence: Contact:\u{a0}\" role=\"email\">{link}</contact>"
      ),
    ),
    (
      repro!("aa_mail_email_label_names_after_the_class"),
      &[&["A. Smith", "affiliation=Univ. X, Y", "email=zz@x.org"][..]][..],
      "<contact name=\"Correspondence to: e-mail: \" role=\"email\">zz@x.org</contact>".to_string(),
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_creators(&xml, creators);
    assert_eq!(email_contacts(&xml), [contact], "{xml}");
  }
}

/// 2604.04847's affiliation line `\textbf{Correspondence:} \href{<address>}{<address>}`: "Correspondence" is no e-mail
/// label, so the address reader reads no address of it; it is a mailbox link line, its e-mail named by its label and
/// the link kept as the source writes it, the block's (the address spells no one, its entry has two carriers). Red
/// (int155a4): the link lost, `<contact …>zz@x.org</contact>`.
#[test]
fn correspondence_link_line_keeps_the_link() {
  let (log, xml) = run(repro!(
    "author_correspondence_link_line_printing_its_address_keeps_the_link"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &["Ann Able", "affiliation=Univ A"],
    &["Bob Baker", "affiliation=Univ B"],
    &["Cid Cole", "affiliation=Univ B"],
    &["Dee Dorn", "affiliation=Univ A"],
    &["email=zz@x.org"],
  ]);
  assert_eq!(
    email_contacts(&xml),
    [
      "<contact name=\"Correspondence:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" \
      href=\"zz@x.org\">zz@x.org</ref></contact>"
    ],
    "{xml}"
  );
}
