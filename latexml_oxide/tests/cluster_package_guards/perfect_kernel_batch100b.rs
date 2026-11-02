//! Red/green guards for batch 100b: one Kind for the addresses printed inside an affiliation-kind entry, and one Link
//! chain both engines run (OXIDIZED_DESIGN_DIVERGENCES #956; FRONTMATTER_SINGLE_ENGINE_PLAN "The structure model";
//! census127 cluster 1, the address family). Code paths: the Kind step at the frontmatter API's one call site after the
//! Cut — a line of addresses, an entry of addresses alone, a marked entry's mark-led line, a note of a labelled address
//! (latexml_engine/src/base_utilities/author_block/affiliations.rs `kind_entry_addresses`); the Link step over each
//! entry's e-mails at the flush (frontmatter/insert.rs `link_entry_addresses`, the chain
//! `email_owners::entry_address_owners`, the frontmatter observer's too); the readers' `|` separator
//! (`ADDRESS_GROUP_SEPARATORS`) and a two-letter surname (`NameKeys`). Each guard converts its repros under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` and asserts no Error or Warning and every creator's name and
//! contacts (role and text) in order, the block's own creator last. Witnesses 2502.01754, 2602.21641, 2307.02047,
//! 1901.09107, 0902.3918, 1909.00276, astro-ph0512555, 1002.3223, cond-mat0202268, 2608.22602.
use latexml::util::test::convert_files_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
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

/// Converts `tex`, checks it has no Error and no Warning and that its creators are `expected` (each its name, then each
/// contact as `role=text`; the block's own creator, nameless, its contacts alone); returns the XML.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&[&str]]) -> String {
  let (log, xml) = convert_files_with(tex, &[], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{xml}");
  xml
}

/// An address line inside one authblk `\affil` (2502.01754): the entry keeps its other lines, each address the author
/// it spells. Red (int127a): `Univ A Town {aable, bbaker, ccole}@a.edu` an affiliation of every author.
#[test]
fn an_address_line_of_an_entry_is_its_authors_emails() {
  assert_creators(repro!("address_family_authblk_address_line"), &[
    &["Ann Able", "affiliation=Univ A Town", "email=aable@a.edu"],
    &["Bob Baker", "affiliation=Univ A Town", "email=bbaker@a.edu"],
    &["Cat Cole", "affiliation=Univ A Town", "email=ccole@a.edu"],
  ]);
}

/// An authblk entry of addresses alone, `\affil[ ]{\texttt{\{…\}@x}}` (2602.21641, 2307.02047): the entry, empty, is
/// their carrier and goes; each address the author it spells. Red (int127a): the brace list the last author's
/// affiliation.
#[test]
fn an_entry_of_addresses_alone_is_no_affiliation() {
  assert_creators(repro!("address_family_authblk_address_entry"), &[
    &["Ann Able", "affiliation=Univ A", "email=aable@a.edu"],
    &["Bob Baker", "affiliation=Univ A", "email=bbaker@a.edu"],
    &["Cat Cole", "affiliation=Univ A", "email=ccole@a.edu"],
  ]);
}

/// Address lines a mark leads in a kernel author block (1901.09107): each the e-mails of the authors showing its mark,
/// with a link line closing the block (the url the block's) and without (the control the frontmatter observer's writer
/// read before). Red (int127a): with the link line, each address line an affiliation of its mark's carriers.
#[test]
fn mark_led_address_lines_are_their_marks_emails() {
  assert_creators(repro!("address_family_marked_address_lines_then_link"), &[
    &["Ann Able", "affiliation=Univ A", "email=aable@a.edu"],
    &["Bob Baker", "affiliation=Univ B", "email=bbaker@b.edu"],
    &["Cat Cole", "affiliation=Univ A", "email=ccole@a.edu"],
    &["url=x.org"],
  ]);
  assert_creators(repro!("address_family_marked_address_lines"), &[
    &["Ann Able", "affiliation=Univ A", "email=aable@a.edu"],
    &["Bob Baker", "affiliation=Univ B", "email=bbaker@b.edu"],
    &["Cat Cole", "affiliation=Univ A", "email=ccole@a.edu"],
  ]);
}

/// A `|` parts local parts before one domain, in a brace list (`\email{\{aable|bbaker\}@a.edu}`, 0902.3918) and with no
/// braces in an institute's line (`\email{aable|bbaker@a.edu}`, 1909.00276), in both readers; each address the author
/// it spells, the class's own label ("E-mail") kept. Red (int127a): the list one contact on a nameless creator.
#[test]
fn a_bar_parts_the_local_parts_of_a_list() {
  for tex in [
    repro!("address_family_llncs_bar_list"),
    repro!("address_family_llncs_inline_bar_list"),
  ] {
    let xml = assert_creators(tex, &[
      &["Ann Able", "affiliation=Univ A", "email=aable@a.edu"],
      &["Bob Baker", "affiliation=Univ A", "email=bbaker@a.edu"],
      &["Cat Cole", "affiliation=Univ B", "email=ccole@b.edu"],
    ]);
    assert_eq!(
      xml
        .matches("<contact name=\"E-mail\u{a0}\" role=\"email\">")
        .count(),
      3,
      "{xml}"
    );
  }
}

/// The Link chain over an entry's addresses (FIDELITY_CHARTER §4): an "Email:" line under a marked entry whose
/// addresses fit no order (astro-ph0512555) gives the spelled one its author and the others once to the block (ruling
/// E, main session 2026-11-02), none left in the entry; one address per carrier goes by order (the control); under a
/// symbol-led entry, an address no carrier spells is every carrier's, before the order. Red (int127a): the e-mail line
/// welded into entry 2 on all four authors; the symbol line's addresses parted by order.
#[test]
fn the_link_chain_over_an_entrys_addresses() {
  let xml = assert_creators(repro!("address_family_email_line_unplaced"), &[
    &[
      "Ann Able 1,2",
      "affiliation=Univ A, Town",
      "email=aable@a.edu",
      "affiliation=Univ B, City",
    ],
    &["Bob Baker 2", "affiliation=Univ B, City"],
    &[
      "Cat Cole 2",
      "affiliation=Univ B, City",
      "email=ccole@b.edu",
    ],
    &["Dan Dee 2", "affiliation=Univ B, City"],
    &["email=zbb@b.edu", "email=ddq@b.edu"],
  ]);
  // (the Kind step's e-mails named by their line's label; Ann's, out of her entry's text, the observer writer's)
  assert_eq!(
    xml
      .matches("<contact name=\"Email:\u{a0}\" role=\"email\">")
      .count(),
    3,
    "{xml}"
  );
  assert_creators(repro!("address_family_email_line_in_order"), &[
    &[
      "Ann Able 1",
      "affiliation=Univ A, Town",
      "email=aable@a.edu",
    ],
    &[
      "Bob Baker 1",
      "affiliation=Univ A, Town",
      "email=office@a.edu",
    ],
    &["Cat Cole 2", "affiliation=Univ B, City"],
  ]);
  assert_creators(repro!("address_family_symbol_line_addresses"), &[
    &[
      "Ann Able †",
      "affiliation=† Univ X, Town",
      "email=alpha@x.org",
      "email=beta@y.org",
    ],
    &["Bob Baker 1", "affiliation=1 Univ Y, City"],
    &[
      "Cat Cole †",
      "affiliation=† Univ X, Town",
      "email=alpha@x.org",
      "email=beta@y.org",
    ],
  ]);
}

/// A two-letter surname spells a local part of its letters alone (`an107@` Weili An's), where the initials of another
/// name are the same letters (Anusuya Nallathambi's "an"): the surname's spelling wins the tie (2608.22602).
#[test]
fn a_two_letter_surname_spells_its_local_part() {
  assert_creators(repro!("address_family_two_letter_surname"), &[
    &["Weili An", "affiliation=Univ P", "email=an107@p.edu"],
    &[
      "Anusuya Nallathambi",
      "affiliation=Univ P",
      "email=anallat@p.edu",
    ],
  ]);
}

/// A note inside an affiliation printing a labelled address alone — REVTeX's `\footnote{Electronic address: …}`
/// (1002.3223), a `\thanks{Corresponding author e-mail: …}` (cond-mat0202268) — is the entry's e-mail, named by the
/// label, and no note. Red (int127a): a document `<pubnote>` each; the `\thanks` text nowhere in the frontmatter.
#[test]
fn an_address_note_in_an_affiliation_is_its_email() {
  let xml = assert_creators(repro!("address_family_revtex_affiliation_footnote"), &[
    &["Ann Able", "affiliation=Univ A, Town", "email=aable@a.edu"],
    &[
      "Bob Baker",
      "affiliation=Univ B, City",
      "email=bbaker@b.edu",
    ],
  ]);
  assert_eq!(
    xml
      .matches("<contact name=\"Electronic address:\u{a0}\" role=\"email\">")
      .count(),
    2,
    "{xml}"
  );
  assert!(!xml.contains("<pubnote"), "{xml}");
  let xml = assert_creators(repro!("address_family_affiliation_thanks_labelled"), &[
    &["Ann Able", "affiliation=Univ A, Town", "email=aable@a.edu"],
    &["Bob Baker", "affiliation=Univ A, Town"],
  ]);
  assert!(
    xml.contains(
      "<contact name=\"Corresponding author e-mail:\u{a0}\" role=\"email\">aable@a.edu</contact>"
    ),
    "{xml}"
  );
}
