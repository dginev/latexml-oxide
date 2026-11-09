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
//! 1901.09107, 0902.3918, 1909.00276, astro-ph0512555, 1002.3223, cond-mat0202268, 2608.22602; review rev100b's
//! 2308.02269 (an ORCID glued to an address line) and 1907.01388 (a symbol carrier another address spells); review
//! rev100b2's 2311.01166 (the writer's copies in print order, the line's font), 1804.05417 and 2410.24095 (the font).
use latexml::util::test::convert_files_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batcht1::red_at_two,
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
    if red_at_two(r#"writes aable@a.edu and bbaker@a.edu as plain text, 2 false Warnings"#) {
      return;
    }
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
  // (an address no one spells goes to the symbol carriers no other address of the entry spells: 1907.01388's `zhutou@`
  // to Tao Zhou alone, review rev100b S1. Red (100b r1): `qz77@` on Ann too)
  assert_creators(
    repro!("address_family_symbol_carrier_spelled_elsewhere"),
    &[
      &[
        "Ann Able *",
        "affiliation=Univ X, Town",
        "email=ann.able@x.org",
      ],
      &["Bob Baker", "affiliation=Univ X, Town"],
      &["Cat Cole *", "affiliation=Univ Y, Town", "email=qz77@y.org"],
    ],
  );
  if red_at_two(
    r#"a false content-check Warning (alpha@x.org, beta@y.org placed more often than printed)"#,
  ) {
    return;
  }
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
/// name are the same letters (Anusuya Nallathambi's "an"): the surname's spelling wins the tie (2608.22602). Not a word
/// of the entry's own text (`ai@` under "AI Lab" and Wei Ai), nor a name particle (`de@` under Ana de Souza): those
/// spell no one, and the chain gives them to the block (review rev100b S3). Red (100b r1): `ai@` on Wei Ai, `de@` on
/// Ana de Souza.
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
  assert_creators(repro!("address_family_entry_word_mailbox"), &[
    &["Wei Ai", "affiliation=AI Lab, Univ X, Town"],
    &["Bob Baker", "affiliation=AI Lab, Univ X, Town"],
    &["email=ai@x.org"],
  ]);
  assert_creators(repro!("address_family_particle_mailbox"), &[
    &["Ana de Souza", "affiliation=Univ X, Town"],
    &["Bob Baker", "affiliation=Univ X, Town"],
    &["email=de@x.org"],
  ]);
  // (a particle is one only before another word of the name, `Thanh Le`'s `le@`; a state code after a comma names no
  // mailbox, "Cambridge, MA 02138" and Jun Ma's `ma@`: review rev100b2 S3, red on 100b r2, both the block's)
  assert_creators(repro!("address_family_surname_particle_state_code"), &[
    &["Thanh Le", "affiliation=Univ X, Town", "email=le@x.org"],
    &["Bob Baker", "affiliation=Univ X, Town"],
    &[
      "Jun Ma",
      "affiliation=Harvard, Cambridge, MA 02138",
      "email=ma@harvard.edu",
    ],
    &["Cat Cole", "affiliation=Harvard, Cambridge, MA 02138"],
  ]);
}

/// A symbol-led line's brace list under several carriers (2311.01166's `$^\ddag$\tt\{kyding,ycwang12,zhxu13,sjwang3,
/// congliu2\}@iflytek.com`): the frontmatter observer's writer copies the unspelled addresses onto the carriers the
/// flush's chain left out (Keyu Ding, Cong Liu), and every carrier's e-mails come in the order the line prints them —
/// the writer's print-order pass orders one span's items by their place in the reading (`plan::reading_order`), not
/// by the order of their texts. Distinct from the Link-chain guards: the writer's copy path. Red (100b r2): Keyu Ding's
/// sjwang3 before ycwang12, Cong Liu's congliu2 first (review rev100b2 B1).
#[test]
fn copied_addresses_keep_the_lines_print_order() {
  let shared = [
    "ycwang12@iflytek.com",
    "zhxu13@iflytek.com",
    "sjwang3@iflytek.com",
  ]
  .map(|a| format!("email={a}"));
  let carrier = |name: &str, before: &[&str], after: &[&str]| -> Vec<String> {
    let mut out = vec![name.to_string()];
    out.extend(before.iter().map(|s| (*s).to_string()));
    out.extend(shared.iter().cloned());
    out.extend(after.iter().map(|s| (*s).to_string()));
    out
  };
  let lab = "affiliation=iFLYTEK Research, China";
  let univ = "affiliation=University of Science and Technology of China";
  let want = [
    carrier(
      "Keyu Ding",
      &[
        univ,
        lab,
        "email=kyding@ustc.edu.cn",
        "email=kyding@iflytek.com",
      ],
      &[],
    ),
    carrier("Yongcan Wang", &[lab], &[]),
    carrier("Zihang Xu", &[lab], &[]),
    carrier("Zhenzhen Jia", &[lab], &[]),
    carrier("Shijin Wang", &[lab], &[]),
    carrier("Cong Liu", &[lab], &["email=congliu2@iflytek.com"]),
    vec![
      "Enhong Chen".to_string(),
      univ.to_string(),
      "email=cheneh@ustc.edu.cn".to_string(),
    ],
  ];
  let want: Vec<Vec<&str>> = want
    .iter()
    .map(|c| c.iter().map(String::as_str).collect())
    .collect();
  let want: Vec<&[&str]> = want.iter().map(Vec::as_slice).collect();
  if red_at_two(
    r#"a false content-check Warning (the listed iflytek addresses placed more often than printed)"#,
  ) {
    return;
  }
  let xml = assert_creators(repro!("address_family_symbol_list_print_order"), &want);
  // (each address in the line's `\tt`, review rev100b2 #2)
  assert_eq!(
    (
      xml.matches("role=\"email\">").count(),
      xml
        .matches("role=\"email\"><text font=\"typewriter\"")
        .count()
    ),
    (22, 22),
    "{xml}"
  );
}

/// The addresses an entry's line prints, written out one e-mail each by the Kind step, keep the font the line printed
/// them in (`email_lines::in_line_font`): a group's declaration, `{\tt \{able, baker\}@x.org}` (1804.05417), and a
/// wrapper round each group of the line, `\texttt{\{cole, dorn\}@y.org}, \texttt{eden@z.org}` (2410.24095), and a
/// declaration repeated before the next address, `{\tt\small fay@w.org, \tt\small \{gil, hal\}@v.org}` (2311.04698),
/// and a font inside a whole-line wrapper, `\small{\texttt{\{ivy, jon\}@u.org}}` (2002.07720). Red (100b r2): every
/// address plain text (review rev100b2 #2, the reviewer's p15).
#[test]
fn written_out_addresses_keep_the_lines_font() {
  let xml = assert_creators(repro!("address_family_line_font"), &[
    &["Ann Able", "affiliation=Univ X, Town", "email=able@x.org"],
    &["Bob Baker", "affiliation=Univ X, Town", "email=baker@x.org"],
    &["Cat Cole", "affiliation=Univ Y, City", "email=cole@y.org"],
    &["Dan Dorn", "affiliation=Univ Y, City", "email=dorn@y.org"],
    &["Eve Eden", "affiliation=Univ Y, City", "email=eden@z.org"],
    &["Fay Fox", "affiliation=Univ Z, Port", "email=fay@w.org"],
    &["Gil Gray", "affiliation=Univ Z, Port", "email=gil@v.org"],
    &["Hal Hill", "affiliation=Univ Z, Port", "email=hal@v.org"],
    &["Ivy Ink", "affiliation=Univ W, Bay", "email=ivy@u.org"],
    &["Jon Jay", "affiliation=Univ W, Bay", "email=jon@u.org"],
  ]);
  // (each e-mail contact one address, in the line's typewriter font)
  let contacts: Vec<&str> = xml
    .split("role=\"email\">")
    .skip(1)
    .filter_map(|c| c.split("</contact>").next())
    .collect();
  let typewriter: Vec<&str> = contacts
    .iter()
    .filter_map(|c| c.strip_prefix("<text font=\"typewriter\""))
    .filter_map(|c| c.split('>').nth(1)?.strip_suffix("</text"))
    .collect();
  assert_eq!(
    typewriter,
    [
      "able@x.org",
      "baker@x.org",
      "cole@y.org",
      "dorn@y.org",
      "eden@z.org",
      "fay@w.org",
      "gil@v.org",
      "hal@v.org",
      "ivy@u.org",
      "jon@u.org"
    ],
    "{xml}"
  );
  assert_eq!(contacts.len(), 10, "{xml}");
}

/// An address line with another call glued after it — an ORCID (2308.02269's `\url{{arim,sue}@…}\orcidID{…}`), a
/// `\thanks` — is no line of addresses alone: the Kind step leaves it in the entry (`prints_addresses_only`), and a
/// contact printing several addresses and more is never written out (`insert::prints_text_only`); each address its
/// author's, each ORCID kept (review rev100b B1). Red (100b r1): `qq1@x.org0000-0002-2701-0271` and the first ORCID
/// lost; `qq1@x.orgCorresponding`.
#[test]
fn an_address_line_holding_another_call_stays_whole() {
  if red_at_two(r#"puts each empty orcid contact before the email the line prints first"#) {
    return;
  }
  let xml = assert_creators(repro!("address_family_line_with_orcid"), &[
    &[
      "Ann Able",
      "affiliation=Univ X, Town",
      "email=qq1@x.org",
      "orcid=",
    ],
    &[
      "Bob Baker",
      "affiliation=Univ X, Town",
      "email=zz2@x.org",
      "orcid=",
    ],
    &[
      "Cat Cole",
      "affiliation=Univ Y, Town",
      "email=cole@y.org",
      "orcid=",
    ],
  ]);
  for (orcid, count) in [("0000-0002-2701-0271", 2), ("0000-0001-6269-9353", 1)] {
    assert_eq!(
      xml
        .matches(&format!("href=\"https://orcid.org/{orcid}\""))
        .count(),
      count,
      "{xml}"
    );
  }
  let xml = assert_creators(repro!("address_family_line_with_thanks"), &[
    &["Ann Able", "affiliation=Univ X, Town", "email=qq1@x.org"],
    &["Bob Baker", "affiliation=Univ X, Town", "email=zz2@x.org"],
  ]);
  assert!(
    xml.contains("<pubnote role=\"thanks\">Corresponding.</pubnote>"),
    "{xml}"
  );
}

/// A note inside an affiliation printing a labelled address alone — REVTeX's `\footnote{Electronic address: …}`
/// (1002.3223), a `\thanks{Corresponding author e-mail: …}` (cond-mat0202268) — is the entry's e-mail, named by the
/// label, and no note (its correspondence words the owner's correspondence note since 107c r2). Red (int127a): a document `<pubnote>` each; the `\thanks` text nowhere in the frontmatter.
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
  // (its label's correspondence words Ann's note before it, the e-mail named by the rest, since 107c r2:
  // FIDELITY_CHARTER §4, review rev107c N4)
  assert!(
    xml.contains(
      "<note class=\"ltx_note_frontmatter ltx_thanks_correspondence\" role=\"thanks\" xml:id=\"id1\">Corresponding author</note>"
    ),
    "{xml}"
  );
  assert!(
    xml.contains("<contact name=\"e-mail:\u{a0}\" role=\"email\">aable@a.edu</contact>"),
    "{xml}"
  );
}
