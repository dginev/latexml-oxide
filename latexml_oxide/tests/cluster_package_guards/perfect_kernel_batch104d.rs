//! Red/green guards for batch 104d: link lists — a block or a corresponding-author line names its owners (census129
//! Bundle 2b). (L1) a cas-sc/cas-dc group's one unlabelled `\affiliation` under no marked author is every author's, as
//! cas prints it under the whole row of names (`_share=authors`, OXIDIZED_DESIGN_DIVERGENCES #901; 2201.00009,
//! 2511.22218, 2106.02306, 2606.24054); two or more are each its run's (`annotate=run`, 2202.06096). (L2) AASTeX 6's `\email` after `\correspondingauthor{X}` before any `\author` is X's
//! (OXIDIZED_DESIGN_DIVERGENCES #963; 2601.19870, 2408.08166, 2206.11167, 2010.09756), and (L3) an address list after
//! it is placed address by address (2511.09481). (L4) llncs institute e-mails placed by spelling: an `Email:` list
//! after `;\@` on the institute's line (2602.18622), one local part in printed braces (2403.13468). Each guard
//! converts its repro and asserts no Error or Warning and every creator whole.
use super::{
  convert_files_with,
  perfect_kernel_batch46::{error_count, warning_count},
};

const PRELOAD: &str = "ar5iv.sty";

/// Every creator of `xml`, whitespace squashed.
fn creators(xml: &str) -> String {
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  xml[start..end].to_string()
}

/// Convert `source` (no shipped files) and return its creators, asserting 0 errors and 0 warnings.
fn clean_creators(source: &str) -> String {
  let (log, xml) = convert_files_with(source, &[], Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators(&xml)
}

/// (L1) cas-sc's `\author{Ann}\affiliation{…}\author{Bob}\author{Cat}`, no marks: the group's one unlabelled
/// affiliation is every author's, before it and after it. Red (int130a): Univ A on Ann Able alone. The control: under
/// `\author[1]`/`\author[2]` the labelled entries go by their marks and the unlabelled Univ C stays the last author's
/// (FIDELITY_CHARTER §4 R1; green before and after). Repros class-bindings/cas_unlabelled_affiliation_is_every_authors,
/// cas_unlabelled_affiliation_under_marked_authors_is_the_last_authors.
#[test]
fn a_cas_unlabelled_affiliation_is_every_authors() {
  let shared = |town: &str| {
    format!(
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town, Land</contact> </creator>{town}"#
    )
  };
  assert_eq!(
    clean_creators(include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/cas_unlabelled_affiliation_is_every_authors.tex"
    )),
    [
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Email: " role="email"><text font="typewriter" xml:id="id1">ann@a.edu</text></contact> "#,
      &shared(" "),
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      &shared(" "),
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      &shared(""),
    ]
    .concat()
  );
  assert_eq!(
    clean_creators(include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/cas_unlabelled_affiliation_under_marked_authors_is_the_last_authors.tex"
    )),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town, Land</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ B, City, Land</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ C, Burg, Land</contact> </creator>"#,
    )
  );
}

/// (L1, brief 104d r2 B1) two or more unlabelled cas entries under no marked author: each is its run's, the authors
/// since the entry before it, as elsarticle reads an unlabelled `\address` (`annotate=run`; 2202.06096). cas-dc's
/// `Ann; Univ A; Bob; Cat; Univ B`: Ann Able Univ A, Bob Baker and Cat Cole Univ B; cas-sc's interleaved `Ann; Univ A;
/// Bob; Univ B`: each author the entry after it. Red (104d r1): every author both; int130a: Bob Baker none in the
/// first, the second green. Two entries after the whole row, `Ann; Bob; Univ A; Univ B`, are that one run's, each
/// author both (main-session decision, review rev104d2 finding 1: the PDF prints "Ann Able, Bob Baker" over both
/// lines; int130a paired them by position). Repros class-bindings/cas_unlabelled_affiliations_are_their_runs,
/// cas_unlabelled_affiliations_interleaved_are_their_authors, cas_unlabelled_affiliations_after_the_row_are_its_runs.
#[test]
fn cas_unlabelled_affiliations_are_their_runs() {
  let entry = |org: &str, town: &str| {
    format!(
      r#"<contact name="Affiliation: " role="affiliation">Univ {org}, {town}, Land</contact> </creator>"#
    )
  };
  assert_eq!(
    clean_creators(include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/cas_unlabelled_affiliations_are_their_runs.tex"
    )),
    [
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      &entry("A", "Town"),
      r#" <creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      &entry("B", "City"),
      r#" <creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      &entry("B", "City"),
    ]
    .concat()
  );
  assert_eq!(
    clean_creators(include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/cas_unlabelled_affiliations_interleaved_are_their_authors.tex"
    )),
    [
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      &entry("A", "Town"),
      r#" <creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      &entry("B", "City"),
    ]
    .concat()
  );
  let both = [
    r#"<contact name="Affiliation: " role="affiliation">Univ A, Town, Land</contact> "#,
    r#"<contact name="Affiliation: " role="affiliation">Univ B, City, Land</contact> </creator>"#,
  ]
  .concat();
  assert_eq!(
    clean_creators(include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/cas_unlabelled_affiliations_after_the_row_are_its_runs.tex"
    )),
    [
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      &both,
      r#" <creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      &both,
    ]
    .concat()
  );
}

/// (L2) aastex631's `\correspondingauthor{Ann Able}\email{ann.able@a.edu}\email{help@x.org}` before the authors: the
/// first address is Ann Able's, beside her correspondent contact; the second, after the flag is spent, is read as
/// before — no author before it, the block's. Red (int130a): both addresses on a nameless creator. Repro
/// class-bindings/aastex631_correspondingauthor_email_is_the_correspondents (census129 Repro A).
#[test]
fn an_aastex6_email_after_correspondingauthor_is_the_correspondents() {
  assert_eq!(
    clean_creators(include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aastex631_correspondingauthor_email_is_the_correspondents.tex"
    )),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town</contact> "#,
      r#"<contact name="Corresponding author: " role="correspondent">Ann Able</contact> "#,
      r#"<contact name="Email: " role="email">ann.able@a.edu</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> "#,
      r#"<creator role="author"> <contact name="Email: " role="email">help@x.org</contact> </creator>"#,
    )
  );
}

/// (L4) llncs's sole institute `Dept.\ of Robotics, Univ A, Town, USA;\@ Email: \email{\{aable,bbaker,ccole\}@a.edu}
/// \\ \email{info@a.edu}`: each listed address the author it spells, named by the printed "Email:" the list was split
/// under (brief 104d r2 S2, FIDELITY_CHARTER §4), the institute's text without the line's `;`; info@a.edu, which no
/// name spells, the block's, with llncs's own name (no label printed before it). Red (int130a): " Email:
/// {aable,bbaker,ccole}@a.edu" whole on a nameless creator; 104d r1: each part named "E-mail ", the label lost. Repro
/// class-bindings/llncs_sole_institute_labelled_email_list_is_spelled.
#[test]
fn an_llncs_labelled_email_list_on_the_institute_line_is_spelled() {
  let institute = r#"<contact name="Affiliation: " role="affiliation">Dept. of Robotics, Univ A, Town, USA</contact> "#;
  let email = |id: u8, address: &str| {
    let name = if id == 4 { "E-mail " } else { "Email: " };
    format!(
      r#"<contact name="{name}" role="email"><text font="typewriter" xml:id="id{id}">{address}</text></contact> </creator>"#
    )
  };
  assert_eq!(
    clean_creators(include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/llncs_sole_institute_labelled_email_list_is_spelled.tex"
    )),
    [
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      institute,
      &email(1, "aable@a.edu"),
      r#" <creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      institute,
      &email(2, "bbaker@a.edu"),
      r#" <creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      institute,
      &email(3, "ccole@a.edu"),
      r#" <creator role="author"> "#,
      &email(4, "info@a.edu"),
    ]
    .concat()
  );
}

/// (L4) llncs's `\institute{Univ A, Town\\ \email{\{bbaker\}@a.edu}\\ \email{\{office\}@a.edu} \and Univ B, City}`
/// under `\inst{1}` on Ann Able and Bob Baker: the lone braced `{bbaker}` spells Bob Baker, `{office}` no one, the
/// block's. Red (int130a): `{bbaker}` on Ann Able, `{office}` on Bob Baker, by place. Repro class-bindings/
/// llncs_institute_lone_braced_address_is_its_spellers.
#[test]
fn an_llncs_lone_braced_address_is_its_spellers() {
  assert_eq!(
    clean_creators(include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/llncs_institute_lone_braced_address_is_its_spellers.tex"
    )),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town</contact> "#,
      r#"<contact name="E-mail " role="email"><text font="typewriter" xml:id="id1">{bbaker}@a.edu</text></contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> "#,
      r#"<creator role="author"> <contact name="E-mail " role="email"><text font="typewriter" xml:id="id2">{office}@a.edu</text></contact> </creator>"#,
    )
  );
}
