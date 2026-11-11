//! Guards for GL1: an e-mail label glued to its address names the line's contact (`email_line_label`,
//! OXIDIZED_DESIGN_DIVERGENCES #956): `Email:zz@x.org` reads as the list `[zz@x.org]`, and the label is split off when
//! the whole line reads the same addresses as the rest behind it, so the address is the content and the label the name,
//! as the spaced `Email: zz@x.org` gives. Guarded here on two routes to `email_line_calls`, the Kind step's address
//! line and an entry's note; the class-named entry's join (aa, REVTeX) is
//! `perfect_kernel_batchmb2::glued_label_line_names_its_contacts`. The affiliation runs' routes name the contact too,
//! but place a glued line on a block creator where the spaced line goes to an author — a separate RED (catalog repros
//! `emulateapj_affil_glued_email_line_{under_one_author,after_affiliations}_owner.tex`).
//!
//! Each converts its repro and asserts the exact Error and Warning counts, every creator's name and contacts in order,
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

/// An address line whose entry the class gives no name: amsart's `\address{Univ A, Town\\ Email:zz@x.org}` is the
/// address "Univ A, Town" and the contact `zz@x.org` named "Email: " (red on int157a: `Email:zz@x.org` in the content,
/// "Email: Email:zz@x.org" in HTML, `mailto:Email:zz@x.org`). Witnesses 2206.00543, astro-ph0001190, hep-ph0411269,
/// hep-ph9606235; elsarticle's unlabelled `\address` takes this path too (catalog repro
/// `elsarticle_address_glued_email_label_names_its_contact.tex`).
#[test]
fn glued_label_names_an_address_line_contact() {
  let (log, xml) = run(repro!("amsart_address_glued_email_label_names_its_contact"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&[
    "Ann Able",
    "address=Univ A, Town",
    "email=zz@x.org",
  ]]);
  assert_eq!(
    email_contacts(&xml),
    ["<contact name=\"Email:\u{a0}\" role=\"email\">zz@x.org</contact>"],
    "{xml}"
  );
}

/// A label glued to a command, not an address: REVTeX's `\altaffiliation{Email:\href{mailto:zz@x.org}{zz@x.org}}`
/// (`split_email_label` ends at the colon, the rest the link) is the link alone named "Email: " (red on int157a: the
/// label before the link in the content). Synthetic (RC-GL probe `o_rv_href`).
#[test]
fn glued_label_names_a_mailto_link_contact() {
  let (log, xml) = run(repro!(
    "revtex_altaffiliation_glued_label_before_mailto_link_names_it"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&[
    "Ann Able",
    "affiliation=Univ A",
    "email=zz@x.org",
  ]]);
  assert_eq!(
    email_contacts(&xml),
    [
      "<contact name=\"Email:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" \
      href=\"mailto:zz@x.org\">zz@x.org</ref></contact>"
    ],
    "{xml}"
  );
}

/// An entry's note, its own e-mail line (kind.rs's footnotes): REVTeX's `\affiliation{Univ A\footnote{Email:zz@x.org}}`
/// gives Ann the contact `zz@x.org` named "Email: ", as the spaced note does (red on int157a: `Email:zz@x.org` in the
/// content). Synthetic (GL1 caller probe `cfoot`; amsart's `\address{…\thanks{E-mail:zz@x.org}}` takes this path).
#[test]
fn glued_label_names_an_entry_note_contact() {
  let (log, xml) = run(repro!(
    "revtex_affiliation_footnote_glued_email_label_names_its_contact"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&[
    "Ann Able",
    "affiliation=Univ A",
    "email=zz@x.org",
  ]]);
  assert_eq!(
    email_contacts(&xml),
    ["<contact name=\"Email:\u{a0}\" role=\"email\">zz@x.org</contact>"],
    "{xml}"
  );
}
