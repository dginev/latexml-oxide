//! Red/green guards for batch 107a (html_feedback #7233; Bundle 5 ADDR-b; OXIDIZED_DESIGN_DIVERGENCES #970). Two code
//! paths. (1) A name's spelling by an address — `NameKeys::of`, the one place every spelling site reads (the author
//! block's e-mail lines, the flush's Link step, the frontmatter observer's `spelled_name`) — gains the conventional
//! transliteration of each letter that has one (`resolver::transliteration`: ä ae, ö oe, ü ue, ß ss, å aa, ø oe, æ ae,
//! œ oe), a letter written with an accent or letter command read as the letter it prints
//! (`name_text::spelling_name_text`); `insert::respell_emails` moves an e-mail to the author it spells as printed, a
//! transliterated spelling counted as one (`NameKeys::as_printed`; user ruling 2026-11-05, FIDELITY_CHARTER §4 #420). (2) The flush's Kind step for an address line
//! (`insert::kind_address_lines`) reads a `{…}@domain` list set over several lines as one line — a break inside a
//! printed brace the line opened (`LineText::read`), the lines a brace opens and a later one closes joined
//! (`entry_lines`) — so its entries go down the one-line form's placement order (FIDELITY_CHARTER §4: the author an entry
//! spells, else the consistent author-order list, else elimination, else the block creator; `email_owners::list_owners`),
//! never glued into an affiliation; a split list it declines is a counted `Warning:frontmatter:address-list`, and text
//! after an `@` with no dot is no domain. Round 2 (review rev107a): a surname's six-letter prefix never reuses the given
//! name's letters, a word's length class is its base spelling's, a literal-vs-transliterated tie stays a tie, and an
//! e-mail moves to the author it spells through the transliteration (user rulings 2026-11-05).
//! Each guard converts its repro and asserts the exact Error and Warning counts and every creator whole.
use super::{
  convert_files_with,
  perfect_kernel_batch46::{error_count, warning_count},
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Every creator of `xml`, whitespace squashed, its `xml:id`s (the order the document numbers its texts in) aside.
fn creators(xml: &str) -> String {
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  let mut out = String::new();
  let mut rest = &xml[start..end];
  while let Some(at) = rest.find(r#" xml:id=""#) {
    out.push_str(&rest[..at]);
    let after = &rest[at + r#" xml:id=""#.len()..];
    rest = &after[after.find('"').map_or(after.len(), |close| close + 1)..];
  }
  out.push_str(rest);
  out
}

/// Converts `tex`, checks its (errors, warnings) are `counts`, and returns its creators.
#[track_caller]
fn creators_with(tex: &str, counts: (usize, usize)) -> String {
  let (log, xml) = convert_files_with(tex, &[], Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), counts, "{log}");
  creators(&xml)
}

/// The two authors under University of Alpha and Cy Coe under Department of Physics, University of Beta, of the split-list
/// repros, `ann` and `bo` their e-mails after the affiliation (`bo` empty for none), `cy` Cy Coe's.
fn alpha_beta(ann: &str, bo: &str, bo_name: &str, cy: &str) -> String {
  let email = |address: &str| {
    if address.is_empty() {
      String::new()
    } else {
      format!(r#"<contact name="Email: " role="email">{address}</contact> "#)
    }
  };
  format!(
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> {}</creator> "#,
      r#"<creator before=" " role="author"> <personname>{}</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> {}</creator> "#,
      r#"<creator before=" " role="author"> <personname>Cy Coe</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Department of Physics, University of Beta</contact> "#,
      r#"{}</creator>"#
    ),
    email(ann),
    bo_name,
    email(bo),
    email(cy)
  )
}

/// A local part spelling its author only through the transliteration: `bo.baecker@` for `Bo Bäcker` in a `{…}@domain`
/// list set over two `\texttt` lines (the reporter's `mo.schaeffler@` for Moritz Schäffler, html_feedback #7233 — the
/// frontmatter observer's reading, `spelled_name`, takes the list out once every entry spells an author); and a
/// one-line list, out of the authors' order, of names written with accent and letter commands only — `B\"acker`,
/// `Wei{\ss}enborn`, `Hj\o rth`, `Fl\aa ten` (the author block's e-mail line reader, `NameKeys::of` over the source
/// tokens). Red (int138a): the split list glued into Cy Coe's affiliation; the one-line list given out by position, each
/// address to the wrong author. Repros sectioning-frontmatter/email_list_umlaut_transliteration_split_red,
/// email_list_accent_command_transliteration.
#[test]
fn a_local_part_spells_its_author_through_the_transliteration() {
  assert_eq!(
    creators_with(
      repro!("email_list_umlaut_transliteration_split_red"),
      (0, 0)
    ),
    alpha_beta(
      r#"<text font="typewriter">ann.able@alpha.edu</text>"#,
      r#"<text font="typewriter">bo.baecker@alpha.edu</text>"#,
      "Bo Bäcker",
      r#"<text font="typewriter">cy.coe@alpha.edu</text>"#,
    )
  );
  let author = |before: &str, name: &str, address: &str| {
    format!(
      concat!(
        r#"<creator {}role="author"> <personname>{}</personname> "#,
        r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> "#,
        r#"<contact name="Email: " role="email"><text font="typewriter">{}@alpha.edu</text></contact> "#,
        r#"</creator>"#
      ),
      before, name, address
    )
  };
  assert_eq!(
    creators_with(repro!("email_list_accent_command_transliteration"), (0, 0)),
    [
      author("", "Bo Bäcker", "bo.baecker"),
      author(r#"before=" " "#, "Gus Weißenborn", "g.weissenborn"),
      author(r#"before=" " "#, "Ida Hjørth", "i.hjoerth"),
      author(r#"before=" " "#, "Jon Flåten", "j.flaaten"),
    ]
    .join(" ")
  );
}

/// A `{…}@domain` list set over several lines is one address list whatever its entries spell, placed as the one-line
/// form is (FIDELITY_CHARTER §4, the main session's ruling on review rev107a finding 4; `email_owners::list_owners`):
/// the author an entry spells; else the consistent author-order list — the twin `\texttt` lines (one font wrapper
/// holding the line break, `LineText::read`) and the domain set on the line after the closing brace (`{…}` /
/// `@alpha.edu`, `domain_goes_on`: 1902.04228), three addresses for three authors, so `xq.zz@`, second, is Bo Baker's
/// (OD #805; `email_list_split_order_only`, every entry unspelled, is the order step's own case — elimination never
/// starts there, review rev107a2 finding 2); else elimination — four addresses for four authors out of
/// their order, `qq77@` the one left beside Bo Baker, the one name left (batch 77p r4; 1808.00588's `kdm@`); else the
/// block creator (OD #159) — three bare lines (each an entry line of its own, `entry_lines` joins them), four addresses
/// for three authors, two unspelled. Red (int138a): the lines glued into the affiliation, 0 warnings; round 1 gave each
/// unspelled entry to the block. The control, the same list on one line: the block's e-mail line reader gives `xq.zz@`
/// to Bo Baker, as on int138a. Repros sectioning-frontmatter/email_list_split_unspelled_entry_glued_red,
/// email_list_domain_on_next_line, email_list_split_unspelled_entry_by_elimination, email_list_split_three_bare_lines,
/// email_list_one_line_unspelled_entry_control.
#[test]
fn a_split_address_list_is_read_whatever_its_entries_spell() {
  // (the block creator's e-mails, in the typewriter font they printed in or none)
  let by_order = alpha_beta(
    r#"<text font="typewriter">ann.able@alpha.edu</text>"#,
    r#"<text font="typewriter">xq.zz@alpha.edu</text>"#,
    "Bo Baker",
    r#"<text font="typewriter">cy.coe@alpha.edu</text>"#,
  );
  assert_eq!(
    creators_with(repro!("email_list_split_unspelled_entry_glued_red"), (0, 0)),
    by_order
  );
}

/// A split `{…}@domain` list the Kind step for an address line declines — a tag after it, `… cy.coe\}@alpha.edu
/// (Ann)`, which no address reader parts — stays as printed in the affiliation and is counted: one
/// `Warning:frontmatter:address-list` (signal integrity: fail toward flagging). Red (int138a): 0 warnings. And no list,
/// no Warning: a line ending `lib\{core\}` and the next opening `@alpha\_lab on Twitter`, an `@` with no domain after
/// it (`insert::opens_with_domain`, review rev107a finding 5; red on round 1: 1 warning). Repros
/// sectioning-frontmatter/email_list_split_tagged_declined_warns, email_handle_after_brace_no_list.
#[test]
fn a_split_address_list_left_unread_is_a_counted_warning() {
  let (log, xml) = convert_files_with(
    repro!("email_list_split_tagged_declined_warns"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(
    log.matches("Warning:frontmatter:address-list").count(),
    1,
    "{log}"
  );
  assert_eq!(
    creators(&xml),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bo Baker</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cy Coe</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Department of Physics, University of Beta<break/>"#,
      r#"<text font="typewriter">{ann.able,xq.zz,<break/>cy.coe}@alpha.edu (Ann)</text></contact> "#,
      r#"</creator>"#
    )
  );
  let affiliation = concat!(
    r#"<contact name="Affiliation: " role="affiliation">University of Alpha, project "#,
    r#"<text font="typewriter">lib{core}<break/></text>@alpha_lab on Twitter</contact>"#
  );
  assert_eq!(
    creators_with(repro!("email_handle_after_brace_no_list"), (0, 0)),
    format!(
      concat!(
        r#"<creator role="author"> <personname>Ann Able</personname> {0} "#,
        r#"<contact name="Email: " role="email"><text font="typewriter">ann.able@alpha.edu</text></contact> "#,
        r#"</creator> "#,
        r#"<creator before=" " role="author"> <personname>Bo Baker</personname> {0} </creator>"#
      ),
      affiliation
    )
  );
}

/// html_feedback #7233's layout: `\and` columns, the `{…}@domain` list set over two `\texttt` lines in the last column,
/// which the author block reads as two contacts of Cy Coe's — the head `{ann.able,mo.schaeffler,` an affiliation, the
/// tail `xq.zz,cy.coe}@alpha.edu` an e-mail of one bogus address. The flush joins the contacts a printed brace spans
/// (`insert::join_split_address_contacts`) and reads them as the one list: `mo.schaeffler@` Moritz Schäffler's through
/// the transliteration, `xq.zz@` the block creator's. Red (int138a): the two bogus contacts on Cy Coe, 0 warnings.
/// Repro sectioning-frontmatter/email_list_split_column_layout_contacts.
#[test]
fn a_split_address_list_read_as_contacts_of_its_own_is_one_list() {
  let author = |before: &str, name: &str, institution: &str, address: &str| {
    format!(
      concat!(
        r#"<creator {}role="author"> <personname>{}</personname> "#,
        r#"<contact name="Affiliation: " role="affiliation">{}</contact> "#,
        r#"<contact name="Email: " role="email"><text font="typewriter">{}@alpha.edu</text></contact> "#,
        r#"</creator>"#
      ),
      before, name, institution, address
    )
  };
  assert_eq!(
    creators_with(repro!("email_list_split_column_layout_contacts"), (0, 0)),
    [
      author("", "Ann Able", "University of Alpha", "ann.able"),
      author(
        r#"before=" " "#,
        "Moritz Schäffler",
        "University of Alpha",
        "mo.schaeffler"
      ),
      author(r#"before=" " "#, "Cy Coe", "University of Beta", "cy.coe"),
      concat!(
        r#"<creator role="author"> <contact name="Email: " role="email">"#,
        r#"<text font="typewriter">xq.zz@alpha.edu</text></contact> </creator>"#
      )
      .to_string(),
    ]
    .join(" ")
  );
}

/// The three authors under University of Alpha of a one-line `{…}@domain` list repro, each with the addresses it holds
/// after the affiliation (`(local part, domain)`), and the block creator's, if any, after them.
fn alpha_three(names: [(&str, Option<&str>); 3], domain: &str, block: Option<&str>) -> String {
  let mut creators: Vec<String> = names
    .iter()
    .enumerate()
    .map(|(at, (name, address))| {
      let email = address.map_or(String::new(), |local| {
        format!(
          r#"<contact name="Email: " role="email"><text font="typewriter">{local}@{domain}</text></contact> "#
        )
      });
      format!(
        concat!(
          r#"<creator {}role="author"> <personname>{}</personname> "#,
          r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> {}</creator>"#
        ),
        if at == 0 { "" } else { r#"before=" " "# },
        name,
        email
      )
    })
    .collect();
  if let Some(local) = block {
    creators.push(format!(
      concat!(
        r#"<creator role="author"> <contact name="Email: " role="email">"#,
        r#"<text font="typewriter">{}@{}</text></contact> </creator>"#
      ),
      local, domain
    ));
  }
  creators.join(" ")
}

/// The transliteration is a second spelling of a name, not a lever on the rules a spelling is judged by (review rev107a
/// findings 1, 3; `email_owners::spelled_owners`, `NameKeys::of`): `mueller@` beside Jörg Müller and Hans Mueller is a
/// genuine tie — an address carries no umlaut, so the literal print is no evidence (user ruling 2026-11-05) — and,
/// neither author order nor elimination deciding (two addresses for three names, two names left), goes to the block;
/// `Wei L\"u`'s "lue" keeps "lu"'s length class, a short surname only a local part of those letters alone spells, so
/// `bluesky@` spells no one — the block's likewise. Red (107a r1): `bluesky@` on Wei Lü. Repros
/// sectioning-frontmatter/email_list_literal_and_transliterated_spelling_tie,
/// email_list_short_surname_transliteration_exact.
#[test]
fn a_transliteration_neither_breaks_a_tie_nor_lifts_a_threshold() {
  assert_eq!(
    creators_with(
      repro!("email_list_literal_and_transliterated_spelling_tie"),
      (0, 0)
    ),
    alpha_three(
      [
        ("Jörg Müller", None),
        ("Hans Mueller", None),
        ("Ann Schmidt", Some("schmidt")),
      ],
      "alpha.de",
      Some("mueller")
    )
  );
  assert_eq!(
    creators_with(
      repro!("email_list_short_surname_transliteration_exact"),
      (0, 0)
    ),
    alpha_three(
      [
        ("Wei Lü", None),
        ("Ann Able", Some("aable")),
        ("Bo Baker", None)
      ],
      "alpha.org",
      Some("bluesky")
    )
  );
}

/// A surname's six-letter prefix is scored only on the local part's letters outside its given name's match (review
/// rev107a finding 2; `email_owners::spelled_owners`): `andrea.rossi@` is Andrea Rossi's (11), not Andrea Andreassen's
/// "andrea" counted twice (12) — pre-existing, red on int138a and 107a r1 —, and `soeren.hansen@` Søren Hansen's, not
/// tied with Søren Sørensen's transliterated "soeren" twice and sent to the block — red on 107a r1. Repros
/// sectioning-frontmatter/email_list_surname_prefix_outside_given_name, email_list_nordic_given_name_prefix.
#[test]
fn a_surname_prefix_never_reuses_the_given_names_letters() {
  assert_eq!(
    creators_with(
      repro!("email_list_surname_prefix_outside_given_name"),
      (0, 0)
    ),
    alpha_three(
      [
        ("Andrea Andreassen", None),
        ("Andrea Rossi", Some("andrea.rossi")),
        ("Ann Able", Some("aable")),
      ],
      "alpha.dk",
      None
    )
  );
  assert_eq!(
    creators_with(repro!("email_list_nordic_given_name_prefix"), (0, 0)),
    alpha_three(
      [
        ("Søren Sørensen", None),
        ("Søren Hansen", Some("soeren.hansen")),
        ("Ann Able", Some("aable")),
      ],
      "alpha.dk",
      None
    )
  );
}
