//! Guards for batch 95c: an e-mail address printed inside an affiliation entry, or on an e-mail line the block sets
//! under an entry, is an e-mail contact of the author it spells, by FIDELITY_CHARTER §4's chain, and the entry keeps
//! the rest of its words (`Resolver::take_out_entry_addresses`; the writer's `Op::CutAddresses`/`Op::Address`, today's
//! item cut by the reading's own address reader, `resolver::address_runs`); one `\email` call printing several
//! addresses is decided address by address by spelling (G3, `Resolver::answer_listed_emails`); today's respell reads
//! names by their base letters, so an address spelling its accented holder stays (G6, `insert::respell_emails`).
//! OXIDIZED_DESIGN_DIVERGENCES #938. Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` under the shadow switch in process and with the binary, eager and
//! `--streaming` (`perfect_kernel_batch92a::assert_reading`), asserts the XML's creators with every contact and the
//! reading's tuples whole, no Error or Warning, the writer's outcome, and that the default mode (the fleet's) writes the
//! same creators. Every repro compiles clean with pdflatex (TL 2025).

use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::set_fm_shadow_override;
use serde_json::Value;

use super::{
  perfect_kernel_batch92a::assert_reading,
  perfect_kernel_batch93a::{default_mode_creators, owned},
  perfect_kernel_batcht1::red_at_two,
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// The kernel block's own entries printing their addresses in their text: a list sharing the domain written without
/// spaces (`(aable,bbaker@x.org)`, 0810.2791's shape — each address its speller's), and one after a label word
/// (`- e-mail: zed@y.edu`, 1909.06145's shape — spelling no one, the entry's only carrier's, `owner:only-carrier`), the
/// label word going with it. Today's held each address inside the affiliation copied on every carrier; the writer cuts
/// today's item and makes each e-mail contact.
#[test]
fn an_address_inside_an_entry_is_its_spellers_email() {
  let tex = repro!("entry_address_inside_an_entry_is_its_speller_email.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X, City", "email=aable@x.org"],
    &[
      "Bob Baker",
      "affiliation=Univ X, City",
      "email=bbaker@x.org",
    ],
    &["Cid Cole", "affiliation=Univ Y, Town", "email=zed@y.edu"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X, City | ruling1:request ruling3:address-out",
    "  c email aable@x.org | ruling1:request owner:entry-address owner:spelled",
    "C Bob Baker [1] | line:names sep:comma",
    "  a Univ X, City | ruling1:request ruling3:address-out",
    "  c email bbaker@x.org | ruling1:request owner:entry-address owner:spelled",
    "C Cid Cole [2] | line:names sep:comma",
    "  a Univ Y, Town | ruling1:request ruling3:address-out",
    "  c email zed@y.edu | ruling1:request owner:entry-address owner:only-carrier",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// E-mail lines under the last entry, which the reading continues the entry with (`line:continued`): a list sharing
/// the domain written with spaces, each part spelling an author (2308.10621's `{\tt hyunjun.jung, p.ruhkamp,
/// b.busam@tum.de}`), and a braced group broken over two lines (2311.18002, 2603.12826). Distinct from the guard above:
/// today's item holds the lines after a line break, in their font wrappers, which the cut removes with the break.
#[test]
fn an_email_line_continuing_an_entry_is_its_spellers_emails() {
  let tex = repro!("entry_address_line_continuing_an_entry_is_email.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X", "email=ann.able@x.org"],
    &[
      "Bob Baker",
      "affiliation=Univ X",
      "affiliation=Univ Y",
      "email=bob.baker@x.org",
      "email=bob.baker@y.org",
    ],
    &["Cid Cole", "affiliation=Univ Y", "email=cid.cole@y.org"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "  c email ann.able@x.org | ruling1:request line:continued owner:entry-address owner:spelled",
    "C Bob Baker [1,2] | line:names sep:comma",
    "  a Univ X | ruling1:request",
    "  a Univ Y | ruling1:request line:continued ruling3:address-out",
    "  c email bob.baker@x.org | ruling1:request line:continued owner:entry-address owner:spelled",
    "  c email bob.baker@y.org | ruling1:request line:continued owner:entry-address owner:spelled",
    "C Cid Cole [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request line:continued ruling3:address-out",
    "  c email cid.cole@y.org | ruling1:request line:continued owner:entry-address owner:spelled",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// An address spelling no author: on a symbol-led line every carrier's (the charter's "every symbol carrier",
/// `owner:carriers`), taken out; under a numbered entry two authors carry, the chain's next step would be the block —
/// the reading cannot tell, so it stays in the entry as today's prints it (the brief's fail-safe; the writer's cut
/// takes only the runs the reading holds as e-mails).
#[test]
fn an_unspelled_entry_address_follows_the_chain_or_stays() {
  let tex = repro!("entry_address_unspelled_by_the_chain.tex");
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=† Univ Z",
      "email=desk@z.org",
      "affiliation=Univ X (office@x.org)",
    ],
    &[
      "Bob Baker",
      "affiliation=† Univ Z",
      "email=desk@z.org",
      "affiliation=Univ X (office@x.org)",
    ],
  ];
  if red_at_two(r#"a false content-check Warning (desk@z.org placed more often than printed)"#) {
    return;
  }
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1,†] | line:names",
    "  a Univ Z | ruling2:symbol-line ruling3:address-out",
    "  a Univ X (office@x.org) | ruling1:request",
    "  c email desk@z.org | ruling2:symbol-line owner:entry-address owner:carriers",
    "C Bob Baker [1,†] | line:names sep:comma",
    "  a Univ Z | ruling2:symbol-line ruling3:address-out",
    "  a Univ X (office@x.org) | ruling1:request",
    "  c email desk@z.org | ruling2:symbol-line owner:entry-address owner:carriers",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// A mark-led line of addresses (2108.13865's `$^1$\{cherian, …\}@merl.com \quad$^2$goncalo.pais@…`): the reading
/// has read it as e-mails since 93a r2; today's old reader made it an affiliation of the label-1 names, which the writer
/// declined. Distinct: today's item holds addresses alone — cut to nothing, it is taken.
#[test]
fn a_mark_led_address_line_today_an_affiliation_is_cut() {
  let tex = repro!("entry_address_mark_led_line_today_an_affiliation.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X", "email=aable@x.org"],
    &["Bob Baker", "affiliation=Univ X", "email=bbaker@x.org"],
    &["Cid Cole", "affiliation=Univ Y", "email=ccole@y.org"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "  c email aable@x.org | ruling1:request owner:spelled",
    "C Bob Baker [1] | line:names sep:gap",
    "  a Univ X | ruling1:request",
    "  c email bbaker@x.org | ruling1:request owner:spelled",
    "C Cid Cole [2] | line:names sep:gap",
    "  a Univ Y | ruling1:request",
    "  c email ccole@y.org | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// API entries printing addresses: revtex's `\affiliation{…, email: aable@y.edu, City}` two authors share
/// (cond-mat0607219: the address Ann Able's, the entry keeping `Ph: 123, City`), and llncs's `\institute` entry with a
/// braced group (2009.00299: each its speller's). Distinct from the block guards: the entry is an API call's content
/// (`api:annotate:run`, `api:labelseq`), not a block line.
#[test]
fn an_api_entrys_address_is_its_spellers_email() {
  let tex = repro!("revtex_affiliation_address_is_its_speller_email.tex");
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=Dept X, Univ Y, Ph: 123, City",
      "email=aable@y.edu",
    ],
    &["Bob Baker", "affiliation=Dept X, Univ Y, Ph: 123, City"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Dept X, Univ Y, Ph: 123, City | api:annotate:run ruling3:address-out",
    "  c email aable@y.edu | api:annotate:run owner:entry-address owner:spelled",
    "C Bob Baker [] | line:names",
    "  a Dept X, Univ Y, Ph: 123, City | api:annotate:run ruling3:address-out",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  let tex = repro!("llncs_institute_address_group_is_its_spellers_emails.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X, UK", "email=ann.able@x.org"],
    &[
      "Bob Baker",
      "affiliation=Univ X, UK",
      "email=bob.baker@x.org",
    ],
    &[
      "Cid Cole",
      "affiliation=Univ Y, Town",
      "email=cid.cole@y.org",
    ],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X, UK | api:labelseq ruling3:address-out",
    "  c email ann.able@x.org | api:labelseq owner:entry-address owner:spelled",
    "C Bob Baker [1] | line:names",
    "  a Univ X, UK | api:labelseq ruling3:address-out",
    "  c email bob.baker@x.org | api:labelseq owner:entry-address owner:spelled",
    "C Cid Cole [2] | line:names",
    "  a Univ Y, Town | api:labelseq ruling3:address-out",
    "  c email cid.cole@y.org | api:labelseq owner:entry-address owner:spelled",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// G3: one `\email` printing several addresses — a list sharing the domain with spaces after the last author
/// (0812.3572; 0901.1641's semicolon list alike), the same list without spaces after each author (1110.2127) and one
/// with an address no author spells (2104.11218's shape: `info@x.org` its call's own author's, `owner:call`) — each
/// address the author's it spells, placed once every name is read (Ann's call spells Bob, read after it).
#[test]
fn an_email_call_of_several_addresses_is_decided_by_spelling() {
  let tex = repro!("revtex_email_call_of_several_addresses_by_spelling.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "email=aable@x.org", "affiliation=Univ X"],
    &["Bob Baker", "email=bbaker@x.org", "affiliation=Univ X"],
    &["Cid Cole", "email=ccole@x.org", "affiliation=Univ X"],
  ];
  if red_at_two(r#"2 false plain-text Warnings: aable@x.org and bbaker@x.org lose their markup"#) {
    return;
  }
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Univ X | api:annotate:run",
    "  c email aable@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "C Bob Baker [] | line:names",
    "  a Univ X | api:annotate:run",
    "  c email bbaker@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "C Cid Cole [] | line:names",
    "  a Univ X | api:annotate:run",
    "  c email ccole@x.org | api:annotate:last ruling3:written-out owner:spelled",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  let tex = repro!("revtex_email_list_after_each_author_by_spelling.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "email=aable@x.org", "affiliation=Univ X"],
    &[
      "Bob Baker",
      "email=bbaker@x.org",
      "email=info@x.org",
      "affiliation=Univ X",
    ],
    &["Cid Cole", "email=ccole@y.org", "affiliation=Univ Y"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Univ X | api:annotate:run",
    "  c email aable@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "  c email aable@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "C Bob Baker [] | line:names",
    "  a Univ X | api:annotate:run",
    "  c email bbaker@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "  c email bbaker@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "  c email info@x.org | api:annotate:last ruling3:written-out owner:call",
    "C Cid Cole [] | line:names",
    "  a Univ Y | api:annotate:run",
    "  c email ccole@y.org | api:annotate:last ruling3:written-out owner:spelled",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// G6 (2410.00275): elsarticle's `\author{Javier Martínez-López}\ead{javier.martinez@…}` keeps its address — today's
/// respell (`insert::respell_emails`) reads each name by its base letters, as an address spells it, so the address
/// spells its holder before Carlos Javier Navarro; after `\author{Ann Able, Bob Baker}` (one call, two people)
/// `\ead{aable@x.org}` still goes to Ann Able, as batch81's AASTeX case. Today's output, the reading alike: the writer's
/// outcome `same` since 97e writes elsarticle's family (`UNDECLARED`).
#[test]
fn an_email_spelling_its_accented_holder_is_not_respelled() {
  let tex = repro!("elsarticle_ead_spelling_its_accented_holder_stays.tex");
  let xml: &[&[&str]] = &[
    &[
      "Javier Martínez-López",
      "email=javier.martinez@x.org",
      "address=Univ X",
    ],
    &[
      "Carlos Javier Navarro",
      "email=carlos@x.org",
      "address=Univ X",
    ],
    &["Ann Able", "address=Univ X", "email=aable@x.org"],
    &["Bob Baker", "address=Univ X"],
  ];
  if red_at_two(r#"unverified: puts Ann Able's email before her address, against the print order"#)
  {
    return;
  }
  let reading = assert_reading(tex, Some(xml), &[
    "C Javier Martínez-López [] | line:names",
    "  a Univ X | api:annotate:run",
    "  c email javier.martinez@x.org | api:annotate:last",
    "C Carlos Javier Navarro [] | line:names",
    "  a Univ X | api:annotate:run",
    "  c email carlos@x.org | api:annotate:last",
    "C Ann Able [] | line:names",
    "  a Univ X | api:annotate:run",
    "  c email aable@x.org | api:annotate:last owner:spelled",
    "C Bob Baker [] | line:names sep:comma",
    "  a Univ X | api:annotate:run",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

// ---- round 2 (review rev95c): an address comes out of an entry only at a list boundary ----

/// The default mode's XML creators, each element whole (its `xml:id`s aside, the whitespace between its tags gone),
/// converted clean.
#[track_caller]
fn default_mode_elements(tex: &str) -> Vec<String> {
  let (log, xml, ()) = convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    || set_fm_shadow_override(None),
    |_| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let end =
        start + xml[start..].find("</creator>").expect("closed creator") + "</creator>".len();
      let mut element = String::new();
      let mut rest = &xml[start..end];
      while let Some(at) = rest.find(" xml:id=\"") {
        element.push_str(&rest[..at]);
        let close = rest[at + 9..].find('"').expect("closed id") + at + 10;
        rest = &rest[close..];
      }
      element.push_str(rest);
      element
        .split('\n')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("")
    })
    .collect()
}

/// B1: `insert::respell_emails` moves an e-mail call to the author its address spells by the names as printed; base
/// letters only keep an address on the author it spells so. amsart's `\author{Anna Smith}…\email{muller-group@x.de}`
/// and elsarticle's `\ead{muller@x.de}` before `\author{J\"org M\"uller}` stay on Anna Smith — they spell Müller only by
/// base letters (FIDELITY_CHARTER §4 #420's example); 2410.00275's `javier.martinez@` stays on Martínez-López, the
/// base-letter veto (`an_email_spelling_its_accented_holder_is_not_respelled`). One code path, two classes' calls.
#[test]
fn an_own_email_spelling_a_coauthor_only_by_base_letters_stays() {
  let tex = repro!("amsart_email_spelling_an_accented_coauthor_stays.tex");
  let xml: &[&[&str]] = &[
    &["Anna Smith", "address=Univ X", "email=muller-group@x.de"],
    &["Jörg Müller", "address=Univ Y", "email=jm@y.de"],
  ];
  assert_reading(tex, Some(xml), &[
    "C Anna Smith [] | line:names",
    "  a Univ X | api:annotate:last",
    "  c email muller-group@x.de | api:annotate:last",
    "C Jörg Müller [] | line:names",
    "  a Univ Y | api:annotate:last",
    "  c email jm@y.de | api:annotate:last",
  ]);
  assert_eq!(default_mode_creators(tex), owned(xml));
  let tex = repro!("elsarticle_ead_spelling_an_accented_coauthor_stays.tex");
  let xml: &[&[&str]] = &[&["Anna Smith", "email=muller@x.de"], &[
    "Jörg Müller",
    "email=jm@y.de",
  ]];
  assert_reading(tex, Some(xml), &[
    "C Anna Smith [] | line:names",
    "  c email muller@x.de | api:annotate:last",
    "C Jörg Müller [] | line:names",
    "  c email jm@y.de | api:annotate:last",
  ]);
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// B2, S4: an address with a plain word on both sides (`write to info@x.org or data@x.org for data`), or a
/// parenthesised tag after it (`Emails: ann@y.org (Ann), bob@y.org (Bob)`), is prose at no list boundary, and a label
/// word with no colon inside the text (`\texttt{mail cc@z.org now}`) no label: each stays in its entry, as today's
/// prints it (the reviewer's s1, s2, s5 — 95c had cut "write to  or  for data", "City,  (Ann),  (Bob)", " now").
#[test]
fn an_address_in_prose_stays_in_its_entry() {
  let tex = repro!("entry_address_in_prose_stays.tex");
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=Univ X, write to info@x.org or data@x.org for data",
    ],
    &[
      "Bob Baker",
      "affiliation=Univ Y, City, Emails: ann@y.org (Ann), bob@y.org (Bob)",
    ],
    &["Cid Cole", "affiliation=Univ Z, mail cc@z.org now"],
  ];
  assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X, write to info@x.org or data@x.org for data | ruling1:request",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y, City, Emails: ann@y.org (Ann), bob@y.org (Bob) | ruling1:request",
    "C Cid Cole [3] | line:names sep:comma",
    "  a Univ Z, mail cc@z.org now | ruling1:request",
  ]);
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// B3: a digit run glued before an address by a comma — a telephone number, an ORCID iD — is no local part of a list:
/// the address alone comes out, the digits stay as printed (95c invented `5551234@x.org`, the affiliation "Univ X,
/// Tel"; the reviewer's s2, s7).
#[test]
fn a_digit_run_before_an_address_is_no_local_part() {
  let tex = repro!("entry_address_after_a_digit_run.tex");
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=Univ X, Tel:5551234",
      "email=info@x.org",
    ],
    &[
      "Bob Baker",
      "affiliation=Univ Y, ORCID 0000-0002-1825-0097",
      "email=zq@y.org",
    ],
  ];
  if red_at_two(r#"leaves zq@y.org inside Bob Baker's affiliation instead of his email"#) {
    return;
  }
  assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X, Tel:5551234 | ruling1:request ruling3:address-out",
    "  c email info@x.org | ruling1:request owner:entry-address owner:only-carrier",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y, ORCID 0000-0002-1825-0097 | ruling1:request ruling3:address-out",
    "  c email zq@y.org | ruling1:request owner:entry-address owner:only-carrier",
  ]);
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// B4 (2111.14566, 1804.04123): an address printed through `\href{mailto:…}{…}` comes out with its link — the e-mail
/// contact holds the `ltx:ref` whole (its `href`, `class`, the font inside it) — and the affiliation holds no empty
/// link. Whole elements.
#[test]
fn a_mailto_link_address_moves_whole_into_its_email() {
  let tex = repro!("entry_address_mailto_link_moves_whole.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "affiliation=Univ X", "email=ann@x.org"], &[
    "Bob Baker",
    "affiliation=Univ Y",
    "email=bob@y.org",
  ]];
  assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request ruling3:address-out",
    "  c email ann@x.org | ruling1:request owner:entry-address owner:spelled",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request ruling3:address-out",
    "  c email bob@y.org | ruling1:request owner:entry-address owner:spelled",
  ]);
  assert_eq!(default_mode_elements(tex), [
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ X</contact><contact name=\"Email:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" \
     font=\"typewriter\" href=\"mailto:ann@x.org\">ann@x.org</ref></contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ Y</contact><contact name=\"Email:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" \
     href=\"mailto:bob@y.org\">bob@y.org</ref></contact></creator>",
  ]);
}

/// B5, S1 (1911.11406): the e-mail contacts made of an entry's addresses are named by the label word the cut took
/// (`E-mail: `, as an e-mail line's label names its contacts; the class's `Email: ` only without one), and several for
/// one owner come out in print order (95c: "Email: ", in the order of their keys). Whole elements. Since 100b the two
/// addresses spell Ann Able, as the witness's `mr\_oboudi@yahoo.com and mr\_oboudi@shirazu.ac.ir` spell their author: two
/// addresses no one spells under one name are no one carrier's (OD #956's only-carrier step, 2403.17706).
#[test]
fn entry_emails_are_named_by_their_label_in_print_order() {
  let tex = repro!("entry_address_label_names_contacts_in_print_order.tex");
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=Univ X",
      "email=zed.able@x.org",
      "email=quux.able@y.org",
    ],
    &["Bob Baker", "affiliation=Univ Y", "email=bob@y.org"],
  ];
  assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request ruling3:address-out",
    "  c email zed.able@x.org | ruling1:request owner:entry-address owner:spelled",
    "  c email quux.able@y.org | ruling1:request owner:entry-address owner:spelled",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request ruling3:address-out",
    "  c email bob@y.org | ruling1:request owner:entry-address owner:spelled",
  ]);
  assert_eq!(default_mode_elements(tex), [
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ X</contact><contact name=\"E-mail: \" role=\"email\">zed.able@x.org</contact><contact \
     name=\"E-mail: \" role=\"email\">quux.able@y.org</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ Y</contact><contact name=\"Email:\u{a0}\" role=\"email\">bob@y.org</contact></creator>",
  ]);
}

/// S2, N1: an unspelled address of an entry two authors carry goes to one by place only within one run — revtex's
/// `\affiliation{Dept X, Univ Y, ann@x.org, Phone 123, office@x.org}` gives `office@x.org` to the block, as the charter
/// does (95c gave it to Bob Baker by place): since 107c the flush's Kind step takes it out of the entry as the block
/// creator's e-mail (the line's tail after a comma; it had stayed in the entry); the cut of `ann@x.org` keeps one space
/// ("Univ Y, Phone 123"; 95c left two). Whole elements.
#[test]
fn an_unspelled_address_is_placed_by_order_only_within_its_run() {
  let tex = repro!("revtex_affiliation_addresses_ordered_within_one_run.tex");
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=Dept X, Univ Y, Phone 123",
      "email=ann@x.org",
    ],
    &["Bob Baker", "affiliation=Dept X, Univ Y, Phone 123"],
    &["email=office@x.org"],
  ];
  assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Dept X, Univ Y, Phone 123, office@x.org | api:annotate:run ruling3:address-out",
    "  c email ann@x.org | api:annotate:run owner:entry-address owner:spelled",
    "C Bob Baker [] | line:names",
    "  a Dept X, Univ Y, Phone 123, office@x.org | api:annotate:run ruling3:address-out",
  ]);
  assert_eq!(default_mode_elements(tex), [
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Dept X, Univ Y, Phone 123</contact><contact name=\"Email:\u{a0}\" \
     role=\"email\">ann@x.org</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Dept X, Univ Y, Phone 123</contact></creator>",
    "<creator role=\"author\"><contact name=\"Email:\u{a0}\" role=\"email\">office@x.org</contact></creator>",
  ]);
}

/// B2, N2 (cond-mat0607219): an address ending a line of revtex's `\affiliation{… \\ Ph: 1; FAX: 2; email: aable@y.edu
/// \\ …}` is at a list boundary in the reading (its lines, `events_lined_text`) and the writer (a line break a line's
/// end) alike; its cut takes the separators before it on its line and keeps the break (95c left "FAX: 2; <break/>").
/// Whole elements.
#[test]
fn an_address_ending_a_line_takes_its_separators_not_the_break() {
  let tex = repro!("revtex_affiliation_address_ending_a_line.tex");
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=Dept X, Univ Y Ph: 1; FAX: 2 Jan. 4, 2005",
      "email=aable@y.edu",
    ],
    &[
      "Bob Baker",
      "affiliation=Dept X, Univ Y Ph: 1; FAX: 2 Jan. 4, 2005",
    ],
  ];
  // (the reading's text parts its pieces with commas, as the lexer reads `;`: "Ph: 1, FAX: 2")
  assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Dept X, Univ Y Ph: 1, FAX: 2 Jan. 4, 2005 | api:annotate:run ruling3:address-out",
    "  c email aable@y.edu | api:annotate:run owner:entry-address owner:spelled",
    "C Bob Baker [] | line:names",
    "  a Dept X, Univ Y Ph: 1, FAX: 2 Jan. 4, 2005 | api:annotate:run ruling3:address-out",
  ]);
  assert_eq!(default_mode_elements(tex), [
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Dept X, Univ Y<break/>Ph: 1; FAX: 2<break/>Jan. 4, 2005</contact><contact name=\"email: \" \
     role=\"email\">aable@y.edu</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Dept X, Univ Y<break/>Ph: 1; FAX: 2<break/>Jan. 4, 2005</contact></creator>",
  ]);
}

// ---- round 3 (review rev95c_r2): the wrapper an address prints in, one-sided boundaries, bare labels ----

/// The default mode's XML creators whole ([`default_mode_elements`], a formula `<M/>`), in process and from the binary,
/// eager and `--streaming`, each converted clean.
#[track_caller]
pub(super) fn assert_elements(tex: &str, want: &[&str]) {
  let math = |elements: Vec<String>| -> Vec<String> {
    elements
      .into_iter()
      .map(|mut e| {
        while let Some(start) = e.find("<Math ") {
          let end = e[start..].find("</Math>").expect("closed formula") + start + "</Math>".len();
          e.replace_range(start..end, "<M/>");
        }
        e
      })
      .collect()
  };
  assert_eq!(math(default_mode_elements(tex)), want, "in process");
  for streaming in [false, true] {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    let output = std::process::Command::new(bin)
      .args([
        "t.tex",
        "--dest",
        "t.xml",
        "--nocomments",
        "--timeout=110",
        "--preload=ar5iv.sty",
        if streaming {
          "--streaming=true"
        } else {
          "--streaming=false"
        },
      ])
      .env_remove("LATEXML_FM_SHADOW")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("t.xml");
    let elements: Vec<String> = xml
      .match_indices("<creator ")
      .map(|(start, _)| {
        let end =
          start + xml[start..].find("</creator>").expect("closed creator") + "</creator>".len();
        let mut element = String::new();
        let mut rest = &xml[start..end];
        while let Some(at) = rest.find(" xml:id=\"") {
          element.push_str(&rest[..at]);
          let close = rest[at + 9..].find('"').expect("closed id") + at + 10;
          rest = &rest[close..];
        }
        element.push_str(rest);
        element
          .split('\n')
          .map(str::trim)
          .collect::<Vec<_>>()
          .join("")
      })
      .collect();
    assert_eq!(math(elements), want, "the binary, streaming {streaming}");
  }
}

/// F1 (0808.1112, `\small Email: \emph{duan@iit.edu}`): an address printed in `\emph{…}` — an inline element wrapping
/// text alone, not only an `ltx:text` or `ltx:ref` — comes out in its font, alone (`Email: \emph{…}`) and inside an
/// enclosing `{\small …}`, and the emptied `ltx:emph` leaves the affiliation (95c-r2: the e-mail plain text, "Univ
/// Y<emph/>"). Whole elements.
#[test]
fn an_address_in_emph_keeps_its_font() {
  let tex = repro!("entry_address_in_emph_keeps_its_font.tex");
  assert_reading(tex, None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request ruling3:address-out",
    "  c email ann.able@x.org | ruling1:request owner:entry-address owner:spelled",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request ruling3:address-out",
    "  c email bob.baker@y.org | ruling1:request owner:entry-address owner:spelled",
  ]);
  if red_at_two(r#"writes the \emph addresses as text font="italic", the emph lost"#) {
    return;
  }
  assert_elements(tex, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ X</contact><contact name=\"Email: \" role=\"email\"><emph \
     font=\"italic\">ann.able@x.org</emph></contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> <text fontsize=\"90%\">Univ Y</text></contact><contact \
     name=\"Email:\u{a0}\" role=\"email\"><emph font=\"italic\" fontsize=\"90%\">bob.baker@y.org</emph></contact></creator>",
  ]);
}

/// F2, F6: a link holding an icon beside the address (`\href{mailto:…}{\rule{1ex}{1ex} ann@…}`, a formula
/// `$\boxtimes$`) is no wrapper the cut takes whole — an icon is no text inside the cut —, so the link and its icon stay
/// once, where they printed, and the e-mail is the address alone (95c-r2 copied the link with its icon into the e-mail:
/// the icon printed twice); the separator before the icon stays, the trim stopping at a piece no text (95c-r2: "Univ
/// X<ref>…</ref>"). Whole elements.
#[test]
fn a_link_with_an_icon_keeps_it_once_and_its_separator() {
  let tex = repro!("entry_address_link_with_an_icon_keeps_it_once.tex");
  assert_reading(tex, None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request ruling3:address-out",
    "  c email ann.able@x.org | ruling1:request owner:entry-address owner:spelled",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y, ⊠ | ruling1:request ruling3:address-out",
    "  c email bob.baker@y.org | ruling1:request owner:entry-address owner:spelled",
  ]);
  if red_at_two(
    r#"unverified: moves the icon link into the email contact and drops the affiliation's ", " separator"#,
  ) {
    return;
  }
  assert_elements(tex, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ X, <ref class=\"ltx_href\" href=\"mailto:ann.able@x.org\"><rule height=\"4.3pt\" \
     width=\"4.3pt\"/></ref></contact><contact name=\"Email:\u{a0}\" role=\"email\">ann.able@x.org</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ Y, <ref class=\"ltx_href\" \
     href=\"mailto:bob.baker@y.org\"><M/></ref></contact><contact name=\"Email:\u{a0}\" \
     role=\"email\">bob.baker@y.org</contact></creator>",
  ]);
}

/// F3: one `\href{mailto:able@x.org}{\{able,baker\}@x.org}` printing two addresses: the e-mail keeps the link whole only
/// for the address it links to (`able@`); `baker@` gets the font alone (95c-r2 put `baker@x.org` inside
/// `mailto:able@x.org`). Whole elements.
#[test]
fn a_group_in_one_link_keeps_the_link_for_its_target_only() {
  let tex = repro!("entry_address_group_in_one_link_keeps_its_target.tex");
  assert_reading(tex, None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request ruling3:address-out",
    "  c email able@x.org | ruling1:request owner:entry-address owner:spelled",
    "C Bob Baker [1] | line:names sep:comma",
    "  a Univ X | ruling1:request ruling3:address-out",
    "  c email baker@x.org | ruling1:request owner:entry-address owner:spelled",
    "C Cat Cole [2] | line:names sep:comma",
    "  a Univ Z | ruling1:request",
  ]);
  if red_at_two(r#"drops the mailto link on able@x.org"#) {
    return;
  }
  assert_elements(tex, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ X</contact><contact name=\"Email:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" \
     href=\"mailto:able@x.org\">able@x.org</ref></contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ X</contact><contact name=\"Email:\u{a0}\" \
     role=\"email\">baker@x.org</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ Z</contact></creator>",
  ]);
}

/// F4: a bare e-mail label word (no colon) mid-line after a list's separator or a period, its address ending the
/// line, is the address's label: it goes with the address and names its contact as printed (95c-r2 left "Univ X,
/// E-mail", "Univ Y. Email", "Univ Z, <text>e-mail</text>"). `\texttt{mail cc@z.org now}` staying whole is
/// `an_address_in_prose_stays_in_its_entry`'s. Whole elements.
#[test]
fn a_bare_label_word_before_a_line_ending_address_goes_with_it() {
  let tex = repro!("entry_address_after_a_bare_label_word.tex");
  assert_reading(tex, None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request ruling3:address-out",
    "  c email ann@x.org | ruling1:request owner:entry-address owner:spelled",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y. | ruling1:request ruling3:address-out",
    "  c email bob@y.org | ruling1:request owner:entry-address owner:spelled",
    "C Cat Cole [3] | line:names sep:comma",
    "  a Univ Z | ruling1:request ruling3:address-out",
    "  c email cole@z.org | ruling1:request owner:entry-address owner:spelled",
  ]);
  assert_elements(tex, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ X</contact><contact name=\"E-mail \" role=\"email\">ann@x.org</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ Y.</contact><contact name=\"Email \" \
     role=\"email\">bob@y.org</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ Z</contact><contact name=\"e-mail \" \
     role=\"email\">cole@z.org</contact></creator>",
  ]);
}

/// F5: one list-boundary side is enough only when the other is no lowercase word without a colon — `see also
/// data@x.org`, `Write to info@y.org`, `write to a@z.org.`, `see sales@w.org, City W` are prose and stay as printed
/// (95c-r2 cut "Univ X, see also", "Univ W, see, City W"); `SPAIN. {…}@upc.edu`, `Correspondence: …` still come out
/// (2509.12185, 2601.07312, the witnesses). Whole elements: today's, the base's.
#[test]
fn an_address_beside_a_prose_word_stays_in_its_entry() {
  let tex = repro!("entry_address_in_one_sided_prose_stays.tex");
  assert_reading(tex, None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X, see also data@x.org | ruling1:request",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y. Write to info@y.org | ruling1:request",
    "C Cat Cole [3] | line:names sep:comma",
    "  a Dept Z, Univ Z, write to a@z.org. | ruling1:request",
    "C Dan Dole [4] | line:names sep:comma",
    "  a Univ W, see sales@w.org, City W | ruling1:request",
  ]);
  assert_elements(tex, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ X, see also data@x.org</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ Y. Write to info@y.org</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Dept Z, Univ Z, write to a@z.org.</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan Dole</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ W, see sales@w.org, City W</contact></creator>",
  ]);
}

/// G3 over a line break (2212.08940): one `\email{ann@x.org;\\ bob@x.org}` is written out address by address — the
/// write-out reads today's contact across its `ltx:break` (`apply::text_leaf`, `leaf_text`) — where 95c-r2's writer
/// declined the paper and so dropped the rest of its work (2212.08940's Mohammed Mouniane lost the `$^{*1}$` address
/// int123a's writer gave him). Distinct from `an_email_call_of_several_addresses_is_decided_by_spelling`: today's
/// contact holds two text pieces. Whole elements.
#[test]
fn an_email_call_over_a_line_break_is_written_out() {
  let tex = repro!("amsart_email_call_of_addresses_over_a_line_break.tex");
  if red_at_two(r#"a false plain-text Warning: ann@x.org loses its markup"#) {
    return;
  }
  let reading = assert_reading(tex, None, &[
    "C Ann Able [] | line:names",
    "  a Univ X | api:annotate:last",
    "  c email ann@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "C Bob Baker [] | line:names",
    "  a Univ Y | api:annotate:last",
    "  c email bob@x.org | api:annotate:last ruling3:written-out owner:spelled",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_elements(tex, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Address:\u{a0}\" \
     role=\"address\">Univ X</contact><contact name=\"Email address: \" role=\"email\"><text \
     font=\"typewriter\">ann@x.org</text></contact></creator>",
    "<creator before=\" and \" role=\"author\"><personname>Bob Baker</personname><contact name=\"Email address: \" \
     role=\"email\"><text font=\"typewriter\">bob@x.org</text></contact><contact name=\"Address:\u{a0}\" \
     role=\"address\">Univ Y</contact></creator>",
  ]);
}

/// A replaced `\author` call before an author block whose entry prints an address: the reading's take-outs name their
/// holders by creator index, renumbered past the replaced call's creators when those are deleted (bundle int124a's
/// A/B: the index ran past the end and the writer panicked, 2011.06172's apa.cls `\author{Author}` before
/// `\threeauthors`; an index still in range would have given the address back to another author). Whole elements.
#[test]
fn a_replaced_author_call_before_an_entry_address_keeps_its_holders() {
  let tex = repro!("entry_address_after_a_replaced_author_call.tex");
  assert_elements(tex, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ X</contact><contact name=\"Email:\u{a0}\" role=\"email\">ann@x.org</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ Y</contact></creator>",
  ]);
}
