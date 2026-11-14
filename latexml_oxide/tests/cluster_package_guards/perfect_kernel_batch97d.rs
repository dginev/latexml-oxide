//! Red/green guards for batch 97d: a collaboration or team named in the author line is a `<contact
//! role="collaboration">` on the names it covers, no creator of its own and no part of a name (FIDELITY_CHARTER §4, user
//! ruling 2026-11-01; frontmatter/classify/collaborations.rs; OXIDIZED_DESIGN_DIVERGENCES #946) — the contact Perl's aastex
//! binding makes of `\collaboration` (aas_support.sty.ltxml:126). A piece of the names line, a line under the names or a
//! heading before them; the names beside it split as the marks or the separators say. Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, asserts the same XML
//! both ways, no Error and no Warning, and every creator whole. One guard per path: the names-line piece of a per-author
//! class (cautious split), the line under marked names (the weld site), a heading and its closing piece (one contact),
//! the pieces put together again (`for the XYZ` + `ABC Collaborations`), a name's bracketed remark (the names before
//! it), the line under unmarked names (an `\and` group's), a heading group, and the words that name none. Round 2
//! (review rev97cd): the contact where its line prints among the name's contacts (D1), an unframed line its address
//! continues and a bare "the" before a team no collaboration (D2), a collaboration broken after "and" one contact (D3),
//! a collaboration opening an `\and` group its own heading (D4), a trailing collaboration group every name's (D5), a
//! PoS speaker's marks and addresses his (D6), and a remark's note reference the name's.
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

/// A `(XYZ Collaboration)` line under revtex's marked names (2410.11607): no part of the last name, where Perl's weld
/// (Base_Utility.pool.ltxml:700-701) put it — each name's collaboration. The line break closing an entry is no part of
/// it (`notes::trim_trailing_separator`, batch 99a; was a trailing `<break/>`).
#[test]
fn collaboration_line_under_marked_names_is_each_names() {
  let xml = both_ways(repro!("author_collaboration_line_welded_into_last_name"));
  let one = r#"<contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="ID"> Institute of One, Town 100000, Country</text></contact>"#;
  let two = r#"<contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="ID"> University of Two, City 200000, Country</text></contact>"#;
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

/// What names no collaboration stays as it was: a team or group line unframed under a name (`Machine Learning Group`,
/// 2205.00756; `Google Brain Team`) is its affiliation; a collaboration word before an institution's (`The ATLAS
/// Collaboration Office, CERN`) an affiliation; a surname holding an institution's word (aastex
/// `M.~Huertas-Company\altaffilmark{1}`, 1212.4143) no collaboration. Since 97f the synthetic line parts at its
/// `\altaffilmark` labels, each name with its own line; the real 1212.4143 (`\author` lines unmarked) is still one
/// creator (the SYNC 97a row's `NON_NAME_WORDS` surname residual).
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
  both_ways(company);
}

/// A collaboration line between a name and its institution (0910.2149's `On Behalf Of The ATLAS Collaboration \\
/// University of Wisconsin`, hep-ex0011032's `Representing the BES Collaboration \\ Institute of High Energy Physics`):
/// the whole creator, its contacts in the order the lines print — the collaboration, then the institution (rev97cd D1:
/// the collaboration came last).
#[test]
fn collaboration_line_is_made_where_it_prints() {
  let tex = repro!("author_collaboration_line_keeps_its_print_order");
  let xml = both_ways(tex);
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>Ann Able</personname> <contact role="collaboration">for the XYZ Collaboration</contact> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator>"#
  ]);
  let representing = tex.replace(
    r"\author{Ann Able \\ for the XYZ Collaboration \\ Univ X}",
    r"\author{Ann Able, Bob Baker \\ Representing the BES Collaboration \\ Univ X \\ E-mail: \texttt{ann.able@x.edu}}",
  );
  assert_ne!(representing, tex);
  let xml = both_ways(&representing);
  let bes = r#"<contact role="collaboration">Representing the BES Collaboration</contact> <contact name="Affiliation: " role="affiliation">Univ X</contact>"#;
  assert_eq!(creators(&xml), [
    format!(r#"<creator role="author"> <personname>Ann Able</personname> {bes} <contact name="E-mail: " role="email"><text font="typewriter" xml:id="id1">ann.able@x.edu</text></contact> </creator>"#).as_str(),
    format!(r#"<creator before=" " role="author"> <personname>Bob Baker</personname> {bes} </creator>"#).as_str(),
  ]);
}

/// A collaboration named unframed whose address goes on under it is an employer's line (2501.13770's
/// `\textit{WIDE Consortium}\\ \.Zabbar\\ Malta`), and a team or group under a name framed by a bare "the" its
/// institution (`The Robotics Group`, `the Applied Mathematics Group`): every line each name's affiliation, no
/// collaboration (rev97cd D2).
#[test]
fn unframed_collaboration_with_its_address_and_the_team_stay_affiliations() {
  let xml = both_ways(repro!(
    "author_unframed_collaboration_with_address_stays_affiliation"
  ));
  let aff =
    |text: &str| format!(r#"<contact name="Affiliation: " role="affiliation">{text}</contact>"#);
  assert_eq!(creators(&xml), [
    format!(
      r#"<creator role="author"> <personname>Ann Able</personname> {} {} {} </creator>"#,
      aff(r#"<text font="italic" xml:id="id1">WIDE Consortium</text>"#),
      aff("Zabbar"),
      aff("Malta")
    )
    .as_str(),
    format!(
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> {} {} </creator>"#,
      aff("The Robotics Group"),
      aff("Massachusetts Institute of Technology")
    )
    .as_str(),
    format!(
      r#"<creator before=" " role="author"> <personname>Cid Cole</personname> {} {} </creator>"#,
      aff("the Applied Mathematics Group"),
      aff("Univ X")
    )
    .as_str(),
  ]);
}
