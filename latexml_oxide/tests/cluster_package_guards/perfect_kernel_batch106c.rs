//! Red/green guards for batch 106c: a tail inside the personname (census132 row 10, NAMETAIL;
//! OXIDIZED_DESIGN_DIVERGENCES #967). Code path: the kernel's `\lx@add@author` (frontmatter_fields.rs `creator_call`),
//! where every author route meets — a membership grade written after the name's comma is the author's `membership`
//! contact (`membership_read_out`, 2107.05093, 2302.11403, 2602.16174); a whole author naming a framed collaboration
//! is the collaboration contact of the authors before it (`whole_name_collaboration`, 1201.5050); a collaboration
//! inside a raw class's name-block command is the names' contact (`wrapped_collaboration_out`, 0907.0548, 1111.2718) —
//! and the name list's own collaboration reading (`name_and_remark`): a remark after the name's notes naming whom the
//! author speaks for (`speaks_for_after_notes`, 1702.03664); and the frontmatter observer's reading of a names line
//! leaves a written grade out of the name (`is_membership_part`, 2009.02373). Round r2 (the bundle A/B's merges): the
//! reading gives a grade back to its name where today's holds no contact of it (writer/mod.rs `give_back_grades`:
//! 1510.03162, 2507.19818), `\lx@add@author` takes a grade out only when the line before it prints none
//! (`holds_grade`), and "for X" is a collaboration only when X's words name a group. Each guard converts its repro and
//! asserts no Error or Warning and every creator whole.
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

/// Every creator of `xml`, whitespace squashed.
fn creators(xml: &str) -> String {
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  xml[start..end].to_string()
}

/// Converts `tex` (beside `files`), checks it converts clean, and returns its creators.
#[track_caller]
fn clean_creators(tex: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = convert_files_with(tex, files, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators(&xml)
}

/// IEEEtran's plain `\author` line with grades written after the names' commas, braced in italics or unbraced (`Bob
/// Baker\textsuperscript{2}, Member, OSA`), a note after the last grade (2107.05093): each grade its author's
/// `membership` contact, the note after it. Red (int134b): personnames "Bob Baker, Member, OSA", "Cat Cole, Fellow,
/// IEEE". Repro sectioning-frontmatter/author_written_membership_grade_is_a_contact.
#[test]
fn a_written_membership_grade_is_the_authors_contact() {
  assert_eq!(
    clean_creators(repro!("author_written_membership_grade_is_a_contact"), &[]),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact role="membership">Member, OSA</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ Y</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      r#"<contact role="membership"><text font="italic" xml:id="id1">Fellow, IEEE</text></contact> "#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_funding" role="thanks" xml:id="id2">Funded.</note> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ X</contact> </creator>"#,
    )
  );
}

/// IEEE name blocks: a list's last name with its grade (2602.16174) and one name the cautious split keeps whole, its
/// grade in a `{\it …}` group (2302.11403) — a different split path, the same contact. Red (int134b): personnames
/// "Cat Cole, Fellow, IEEE", "Dan Dee, Senior Member, IEEE". Repro
/// sectioning-frontmatter/author_block_written_membership_grade_is_a_contact.
#[test]
fn a_written_membership_grade_in_a_name_block_is_the_authors_contact() {
  assert_eq!(
    clean_creators(
      repro!("author_block_written_membership_grade_is_a_contact"),
      &[]
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      r#"<contact role="membership"><text font="italic" xml:id="id1">Fellow, IEEE</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Dan Dee</personname> "#,
      r#"<contact role="membership"><text font="italic" xml:id="id2">Senior Member, IEEE</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ Y</contact> </creator>"#,
    )
  );
}

/// elsarticle's `\author{On behalf of the XYZ Consortium}` after its authors (1201.5050): their collaboration contact,
/// no creator. Red (int134b): a third creator "On behalf of the XYZ Consortium". Repro
/// sectioning-frontmatter/elsarticle_author_on_behalf_of_collaboration_is_a_contact.
#[test]
fn an_author_on_behalf_of_a_collaboration_is_the_authors_contact() {
  assert_eq!(
    clean_creators(
      repro!("elsarticle_author_on_behalf_of_collaboration_is_a_contact"),
      &[]
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Address: " role="address">Univ X</contact> "#,
      r#"<contact role="collaboration">On behalf of the XYZ Consortium</contact> "#,
      r#"<contact name="Corresponding author: " role="correspondent">Corresponding author.</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact name="Address: " role="address">Univ X</contact> "#,
      r#"<contact role="collaboration">On behalf of the XYZ Consortium</contact> </creator>"#,
    )
  );
}

/// A raw class's name-block command round the names, the collaboration on a line of its own after the last name's
/// `\\` (icrc2009, 0907.0548): every name's collaboration contact, and each name its affiliation by its mark. Red
/// (int134b): "Cat Cole<break/>on behalf of the XYZ-LAT Collaboration<sup>2</sup>", Ann Able holding both affiliations.
/// Repro sectioning-frontmatter/author_raw_name_block_collaboration_line_is_a_contact.
#[test]
fn a_collaboration_line_in_a_raw_name_block_is_the_names_contact() {
  let collaboration = |id: u32| {
    format!(
      r#"<contact role="collaboration">on behalf of the <emph font="italic" xml:id="id{id}">XYZ</emph>-LAT Collaboration</contact> "#
    )
  };
  assert_eq!(
    clean_creators(
      repro!("author_raw_name_block_collaboration_line_is_a_contact"),
      &[]
    ),
    [
      r#"<creator role="author"> <personname>Ann Able</personname> "#.to_string(),
      collaboration(1),
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Italy</contact> </creator> "#
        .to_string(),
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#.to_string(),
      collaboration(2),
      r#"<contact name="Affiliation: " role="affiliation">Univ B, Italy</contact> </creator> "#
        .to_string(),
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#.to_string(),
      collaboration(3),
      r#"<contact name="Affiliation: " role="affiliation">Univ B, Italy</contact> </creator>"#
        .to_string(),
    ]
    .concat()
  );
}

/// A raw class's name-block command holding one name and its remark after the marks (icrc2009, 1111.2718): the
/// author's collaboration contact, the two affiliations the marks name. Red (int134b): "A.B. de Able for the XYZ
/// collaboration<sup>2</sup>", one affiliation holding both lines. Repro
/// sectioning-frontmatter/author_raw_name_block_collaboration_remark_is_a_contact.
#[test]
fn a_collaboration_remark_in_a_raw_name_block_is_the_authors_contact() {
  assert_eq!(
    clean_creators(
      repro!("author_raw_name_block_collaboration_remark_is_a_contact"),
      &[]
    ),
    concat!(
      r#"<creator role="author"> <personname>A.B. de Able</personname> "#,
      r#"<contact role="collaboration">for the XYZ collaboration.</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, USA</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">now at Univ B, UK</contact> </creator>"#,
    )
  );
}
