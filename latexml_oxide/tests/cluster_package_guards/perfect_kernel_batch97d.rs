//! Red/green guards for batch 97d: a collaboration or team named in the author line is a `<contact
//! role="collaboration">` on the names it covers, no creator of its own and no part of a name (FIDELITY_CHARTER §4, user
//! ruling 2026-11-01; author_block/collaborations.rs; OXIDIZED_DESIGN_DIVERGENCES #946) — the contact Perl's aastex
//! binding makes of `\collaboration` (aas_support.sty.ltxml:126). A piece of the names line, a line under the names or a
//! heading before them; the names beside it split as the marks or the separators say. Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, asserts the same XML
//! both ways, no Error and no Warning, and every creator whole. One guard per path: the names-line piece of a per-author
//! class (cautious split), the line under marked names (the weld site), a heading and its closing piece (one contact),
//! the pieces put together again (`for the XYZ` + `ABC Collaborations`), a name's bracketed remark (the names before
//! it), the line under unmarked names (an `\and` group's), a heading group, and the words that name none.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch77d::creators,
  perfect_kernel_batch89a::cli_conversion,
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

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error and no Warning and both give the
/// same XML, and returns its squashed XML.
#[track_caller]
fn both_ways(tex: &str) -> String {
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (streamed_log, streamed) = cli_conversion(tex, true);
  assert_eq!(
    (error_count(&streamed_log), warning_count(&streamed_log)),
    (0, 0),
    "{streamed_log}"
  );
  assert_eq!(eager, streamed);
  squash(&eager)
}

/// revtex's names line ending in a team, `… and the GAPS Team` (1708.04166, 15 names one creator): two creators, each
/// with the team as its collaboration and the institute its mark names; the marks stay on the names, as the binding
/// keeps them. A team listed with "and" and marked, `… and LHD experimental group$^{1}$` (1711.07097, 11 names one
/// creator and the merge Error): the same two, the mark no part of the contact.
#[test]
fn team_piece_of_a_names_line_is_a_collaboration() {
  let tex = repro!("author_names_line_team_piece_is_a_collaboration");
  let person = |n: &str, before: &str, mark: &str, id: &str, team: &str, univ: &str| {
    format!(
      r#"<creator {before}role="author"> <personname>{n}<sup xml:id="{id}"><text font="italic" xml:id="{id}.1">{mark}</text></sup></personname> <contact role="collaboration">{team}</contact> <contact name="Affiliation: " role="affiliation">{univ}</contact> </creator>"#
    )
  };
  let xml = both_ways(tex);
  assert_eq!(creators(&xml), [
    person("Ann Able", "", "1", "id1", "the GAPS Team", "Univ A").as_str(),
    person(
      "Bob Baker",
      r#"before=", " "#,
      "2",
      "id2",
      "the GAPS Team",
      "Univ B"
    )
    .as_str(),
  ]);
  let listed = tex.replace("and the GAPS Team}", "and LHD experimental group$^{1}$}");
  assert_ne!(listed, tex);
  let xml = both_ways(&listed);
  assert_eq!(creators(&xml), [
    person(
      "Ann Able",
      "",
      "1",
      "id1",
      "LHD experimental group",
      "Univ A"
    )
    .as_str(),
    person(
      "Bob Baker",
      r#"before=", " "#,
      "2",
      "id2",
      "LHD experimental group",
      "Univ B"
    )
    .as_str(),
  ]);
}

/// A `(XYZ Collaboration)` line under revtex's marked names (2410.11607): no part of the last name, where Perl's weld
/// (Base_Utility.pool.ltxml:700-701) put it — each name's collaboration.
#[test]
fn collaboration_line_under_marked_names_is_each_names() {
  let xml = both_ways(repro!("author_collaboration_line_welded_into_last_name"));
  let one = r#"<contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="ID"> Institute of One, Town 100000, Country<break/></text></contact>"#;
  let two = r#"<contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="ID"> University of Two, City 200000, Country<break/></text></contact>"#;
  let at = |contact: &str, id: &str| contact.replace("ID", id);
  let xyz = r#"<contact role="collaboration">(XYZ Collaboration)</contact>"#;
  assert_eq!(creators(&xml), [
    format!(r#"<creator role="author"> <personname>A. Able</personname> {xyz} {} </creator>"#, at(one, "id1")).as_str(),
    format!(
      r#"<creator before=" " role="author"> <personname>B. Baker</personname> {xyz} {} <contact name="Affiliation: " role="affiliation"> Also at Laboratory A, Village 300000, Country</contact> </creator>"#,
      at(two, "id2")
    )
    .as_str(),
    format!(
      r#"<creator before=" " role="author"> <personname>C. Cole</personname> {xyz} {} {} </creator>"#,
      at(one, "id3"),
      at(two, "id4")
    )
    .as_str(),
  ]);
}

/// BESIII's heading over the names and the same collaboration closing them on the names line (2212.06489, 582 names: a
/// creator "BESIII Collaboration", one creator holding the names, the merge Error): each name once its collaboration,
/// led by the separator the line prints.
#[test]
fn collaboration_heading_and_its_closing_piece_are_one_contact() {
  let xml = both_ways(repro!("author_collaboration_heading_and_closing_line_once"));
  let one = r#"<contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="ID"> Institute of One, Town 100000, Country<break/></text></contact>"#;
  let two = r#"<contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="ID"> University of Two, City 200000, Country<break/></text></contact>"#;
  let at = |contact: &str, id: &str| contact.replace("ID", id);
  let xyz = r#"<contact role="collaboration">(XYZ Collaboration)</contact>"#;
  assert_eq!(creators(&xml), [
    format!(r#"<creator role="author"> <personname>A. Able</personname> {xyz} {} </creator>"#, at(one, "id1")).as_str(),
    format!(
      r#"<creator before=", " role="author"> <personname>B. Baker</personname> {xyz} {} <contact name="Affiliation: " role="affiliation"> Also at Laboratory A, Village 300000, Country</contact> </creator>"#,
      at(two, "id2")
    )
    .as_str(),
    format!(
      r#"<creator before=", " role="author"> <personname>C. Cole</personname> {xyz} {} {} </creator>"#,
      at(one, "id3"),
      at(two, "id4")
    )
    .as_str(),
  ]);
}

/// `… Bob Baker$^{2}$ for the XYZ and ABC Collaborations` (dev census c3): the pieces the names split parted at the
/// marks and at "and" are one collaboration again, each name's — not the creators "for the XYZ" and "ABC
/// Collaborations".
#[test]
fn names_speaking_for_two_collaborations_carry_one_contact() {
  let xml = both_ways(repro!("author_names_for_two_collaborations_is_one_contact"));
  let collaboration =
    r#"<contact role="collaboration">for the XYZ and ABC Collaborations</contact>"#;
  assert_eq!(creators(&xml), [
    format!(r#"<creator role="author"> <personname>Ann Able</personname> {collaboration} <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator>"#).as_str(),
    format!(r#"<creator before=" " role="author"> <personname>Bob Baker</personname> {collaboration} <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#).as_str(),
  ]);
}

/// A bracketed collaboration after a name (0910.4381's `J.~Zapatero$^{5}$ (The MAGIC Collaboration), A.~Falcone…`):
/// the name's remark, covering the names before it — A. Able, B. Baker, C. Cole — and not D. Dorn after it.
#[test]
fn bracketed_collaboration_covers_the_names_before_it() {
  let xml = both_ways(repro!(
    "author_name_bracketed_collaboration_covers_names_before"
  ));
  let xyz = r#"<contact role="collaboration">(The XYZ Collaboration)</contact>"#;
  let person = |before: &str, name: &str, collaboration: &str, univ: &str| {
    format!(
      r#"<creator {before}role="author"> <personname>{name}</personname> {collaboration}<contact name="Affiliation: " role="affiliation">{univ}</contact> </creator>"#
    )
  };
  let xyz = format!("{xyz} ");
  assert_eq!(creators(&xml), [
    person("", "A. Able", &xyz, "Univ A").as_str(),
    person(r#"before=" " "#, "B. Baker", &xyz, "Univ B").as_str(),
    person(r#"before=" " "#, "C. Cole", &xyz, "Univ A").as_str(),
    person(r#"before=" " "#, "D. Dorn", "", "Univ C").as_str(),
  ]);
}

/// aastex's `\\ (The Nearby Supernova Factory)` under its names (1511.01102: every name's affiliation): each name's
/// collaboration beside the alternate affiliation its mark names. An article `\and` group's line under its one name is
/// that name's alone (`Ann Able\\ for the ATLAS Collaboration \and Bob Baker\\ Univ B`).
#[test]
fn collaboration_line_under_unmarked_names_covers_its_group() {
  let xml = both_ways(repro!("aastex_collaboration_line_under_names_is_a_contact"));
  let person = |before: &str, name: &str, univ: &str| {
    format!(
      r#"<creator {before}role="author"> <personname>{name}</personname> <contact role="collaboration">(The Nearby Supernova Factory)</contact> <contact name="Alternate Affiliation: " role="altaffiliation">{univ}</contact> </creator>"#
    )
  };
  assert_eq!(creators(&xml), [
    person("", "A. Able", "Univ A").as_str(),
    person(r#"before=" " "#, "B. Baker", "Univ B").as_str(),
    person(r#"before=" " "#, "C. Cole", "Univ A").as_str(),
  ]);
  let grouped = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\\\ for the ATLAS \
                 Collaboration \\and Bob Baker\\\\ Univ B}\n\\maketitle\nBody.\n\\end{document}\n";
  let xml = both_ways(grouped);
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>Ann Able</personname> <contact role="collaboration">for the ATLAS Collaboration</contact> </creator>"#,
    r#"<creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#,
  ]);
}

/// An `\and` group that is only a collaboration's name before the names (1912.05868's `\small H.E.S.S. Collaboration
/// \and H.~Abdalla …` shape): a heading over every name after it, no creator. A call naming nothing else keeps it the
/// author it is (`\author{The ATLAS Collaboration}`).
#[test]
fn collaboration_heading_group_covers_the_names_after_it() {
  let tex = repro!("author_collaboration_heading_group_covers_names_after");
  let xml = both_ways(tex);
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact role="collaboration">XYZ Collaboration</contact> </creator>"#,
    r#"<creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <contact role="collaboration">XYZ Collaboration</contact> </creator>"#,
  ]);
  let alone = tex.replace(
    r"\author{XYZ Collaboration \and Ann Able\\ Univ A \and Bob Baker\\ Univ B}",
    r"\author{The ATLAS Collaboration}",
  );
  assert_ne!(alone, tex);
  let xml = both_ways(&alone);
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>The ATLAS Collaboration</personname> </creator>"#
  ]);
}

/// What names no collaboration stays as it was: a team or group line unframed under a name (`Machine Learning Group`,
/// 2205.00756; `Google Brain Team`) is its affiliation; a collaboration word before an institution's (`The ATLAS
/// Collaboration Office, CERN`) an affiliation; a surname holding an institution's word (aastex
/// `M.~Huertas-Company\altaffilmark{1}`, 1212.4143) no collaboration — its line one creator as 97a leaves it (pinned,
/// not a target: the SYNC 97a row's `NON_NAME_WORDS` surname residual).
#[test]
fn team_words_and_company_surnames_name_no_collaboration() {
  let xml = both_ways(repro!("author_team_and_group_lines_stay_affiliations"));
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Machine Learning Group</contact> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator>"#,
    r#"<creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Google Brain Team</contact> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#,
  ]);
  let office = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{1}$, Bob Baker$^{2}$\\\\ \
                $^1$Univ A\\\\ $^2$The ATLAS Collaboration Office, CERN}\n\\maketitle\nBody.\n\\end{document}\n";
  let xml = both_ways(office);
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator>"#,
    r#"<creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">The ATLAS Collaboration Office, CERN</contact> </creator>"#,
  ]);
  let company = "\\documentclass{aastex}\n\\begin{document}\n\\title{T}\n\\author{M.~Huertas-Company\\altaffilmark{1}, \
                 B.~Baker\\altaffilmark{2}}\n\\altaffiltext{1}{Univ A}\n\\altaffiltext{2}{Univ B}\n\\maketitle\nBody.\n\
                 \\end{document}\n";
  let xml = both_ways(company);
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>M. Huertas-Company, B. Baker</personname> <contact name="Alternate Affiliation: " role="altaffiliation">Univ A</contact> <contact name="Alternate Affiliation: " role="altaffiliation">Univ B</contact> </creator>"#
  ]);
}

/// A heading's colon is the line's own, outside every group (2407.21626's `MeerKLASS Collaboration\thanks{Corresponding
/// authors:}: Matilde …`, 2601.03873's `\parbox{\textwidth}{\Large{ MAGIC Collaboration: K.~Abe$^{1}$, …}}`: cut inside
/// the note or the box, the braces did not close — a Fatal): the names each with the heading's collaboration, the note
/// a thanks pubnote; the line inside a box no heading of the line's, read as before.
#[test]
fn a_heading_colon_is_the_lines_own() {
  let tex = repro!("author_collaboration_heading_colon_outside_groups");
  let xml = both_ways(tex);
  let xyz = r#"<contact role="collaboration">XYZ Collaboration</contact>"#;
  assert_eq!(creators(&xml), [
    format!(r#"<creator role="author"> <personname>Ann Able</personname> {xyz} <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator>"#).as_str(),
    format!(r#"<creator before=" " role="author"> <personname>Bob Baker</personname> {xyz} <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#).as_str(),
  ]);
  assert!(
    xml.contains(r#"<pubnote role="thanks">Corresponding authors: Ann Able</pubnote>"#),
    "{xml}"
  );
  let boxed = tex.replace(
    r"\author{XYZ Collaboration\thanks{Corresponding authors: Ann Able}: Ann Able$^{1}$, Bob Baker$^{2}$\\",
    r"\author{\parbox{\textwidth}{\Large{ XYZ Collaboration: Ann Able$^{1}$, Bob Baker$^{2}$}}\\",
  );
  assert_ne!(boxed, tex);
  let xml = both_ways(&boxed);
  assert!(!xml.contains(r#"role="collaboration""#), "{xml}");
  assert_eq!(creators(&xml).len(), 2, "{xml}");
}
