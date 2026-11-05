//! Red/green guards for batch 107a (html_feedback #7233; Bundle 5 ADDR-b; OXIDIZED_DESIGN_DIVERGENCES #970). Two code
//! paths. (1) A name's spelling by an address — `NameKeys::of`, the one place every spelling site reads (the author
//! block's e-mail lines, the flush's Link step, the frontmatter observer's `spelled_name`) — gains the conventional
//! transliteration of each letter that has one (`resolver::transliteration`: ä ae, ö oe, ü ue, ß ss, å aa, ø oe, æ ae,
//! œ oe), a letter written with an accent or letter command read as the letter it prints
//! (`name_text::spelling_name_text`); `insert::respell_emails` still moves an author's own e-mail only to the author it
//! spells as printed (`NameKeys::as_printed`, FIDELITY_CHARTER §4 #420). (2) The flush's Kind step for an address line
//! (`insert::kind_address_lines`) reads a `{…}@domain` list set over several lines as one line — a break inside a
//! printed brace the line opened (`LineText::read`), the lines a brace opens and a later one closes joined
//! (`entry_lines`) — so its entries go down FIDELITY_CHARTER §4's chain (the author an entry spells, else the block
//! creator), never glued into an affiliation; a split list it declines is a counted `Warning:frontmatter:address-list`.
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

/// FIDELITY_CHARTER §4 #420's own example: Anna Smith's own `\email{mueller-group@x.de}` beside `\author{J\"org
/// M\"uller}`, whom it spells only through the transliteration (ü ue), stays hers — `insert::respell_emails` moves an
/// author's own e-mail only to the author it spells as printed (`NameKeys::as_printed`); its sibling
/// `perfect_kernel_batch95c::an_own_email_spelling_a_coauthor_only_by_base_letters_stays` pins `muller-group@`. Green on
/// int138a too (base letters alone gave "muller"); the transliteration keys must not move it. Repro
/// sectioning-frontmatter/amsart_email_transliterated_coauthor_stays.
#[test]
fn an_own_email_spelling_a_coauthor_only_through_the_transliteration_stays() {
  assert_eq!(
    creators_with(repro!("amsart_email_transliterated_coauthor_stays"), (0, 0)),
    concat!(
      r#"<creator role="author"> <personname>Anna Smith</personname> "#,
      r#"<contact name="Address: " role="address">Univ X</contact> "#,
      r#"<contact name="Email address: " role="email"><text font="typewriter">mueller-group@x.de</text></contact> "#,
      r#"</creator> "#,
      r#"<creator before=" and " role="author"> <personname>Jörg Müller</personname> "#,
      r#"<contact name="Address: " role="address">Univ Y</contact> "#,
      r#"<contact name="Email address: " role="email"><text font="typewriter">jm@y.de</text></contact> "#,
      r#"</creator>"#
    )
  );
}

/// A `{…}@domain` list set over several lines is one address list whatever its entries spell: an entry spelling an
/// author goes to that author, one spelling no one to the block creator (FIDELITY_CHARTER §4 "else the block", user
/// 2026-11-05: block addresses live on OD #159's block creator) — the twin `\texttt` lines, one font wrapper holding
/// the line break (`LineText::read`), and three bare lines with two unspelled entries, each line an entry line of its
/// own (`entry_lines` joins them), and the domain set on the line after the closing brace (`{…}` / `@alpha.edu`,
/// `domain_goes_on`: 1902.04228). Red (int138a): the lines glued into Cy Coe's affiliation, 0 warnings. The control,
/// the same list on one line: the block's e-mail line reader gives `xq.zz@`, the one address left beside the one name
/// left, to Bo Baker by elimination (user decision 2026-10-20, batch 77p r4), as on int138a. Repros
/// sectioning-frontmatter/email_list_split_unspelled_entry_glued_red, email_list_split_three_bare_lines,
/// email_list_domain_on_next_line, email_list_one_line_unspelled_entry_control.
#[test]
fn a_split_address_list_is_read_whatever_its_entries_spell() {
  // (the block creator's e-mails, in the typewriter font they printed in or none)
  let block = |addresses: &[&str], typewriter: bool| {
    let contacts: Vec<String> = addresses
      .iter()
      .map(|address| {
        if typewriter {
          format!(
            r#"<contact name="Email: " role="email"><text font="typewriter">{address}@alpha.edu</text></contact>"#
          )
        } else {
          format!(r#"<contact name="Email: " role="email">{address}@alpha.edu</contact>"#)
        }
      })
      .collect();
    format!(
      r#" <creator role="author"> {} </creator>"#,
      contacts.join(" ")
    )
  };
  assert_eq!(
    creators_with(repro!("email_list_split_unspelled_entry_glued_red"), (0, 0)),
    alpha_beta(
      r#"<text font="typewriter">ann.able@alpha.edu</text>"#,
      "",
      "Bo Baker",
      r#"<text font="typewriter">cy.coe@alpha.edu</text>"#,
    ) + block(&["xq.zz"], true).as_str()
  );
  assert_eq!(
    creators_with(repro!("email_list_domain_on_next_line"), (0, 0)),
    alpha_beta(
      r#"<text font="typewriter">ann.able@alpha.edu</text>"#,
      "",
      "Bo Baker",
      r#"<text font="typewriter">cy.coe@alpha.edu</text>"#,
    ) + block(&["xq.zz"], true).as_str()
  );
  assert_eq!(
    creators_with(repro!("email_list_split_three_bare_lines"), (0, 0)),
    alpha_beta("ann.able@alpha.edu", "", "Bo Baker", "cy.coe@alpha.edu")
      + block(&["xq.zz", "yy.ww"], false).as_str()
  );
  assert_eq!(
    creators_with(
      repro!("email_list_one_line_unspelled_entry_control"),
      (0, 0)
    ),
    alpha_beta(
      r#"<text font="typewriter">ann.able@alpha.edu</text>"#,
      r#"<text font="typewriter">xq.zz@alpha.edu</text>"#,
      "Bo Baker",
      r#"<text font="typewriter">cy.coe@alpha.edu</text>"#,
    )
  );
}

/// A split `{…}@domain` list the Kind step for an address line declines — a tag after it, `… cy.coe\}@alpha.edu
/// (Ann)`, which no address reader parts — stays as printed in the affiliation and is counted: one
/// `Warning:frontmatter:address-list` (signal integrity: fail toward flagging). Red (int138a): 0 warnings. Repro
/// sectioning-frontmatter/email_list_split_tagged_declined_warns.
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
