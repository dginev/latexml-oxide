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
  perfect_kernel_batcht1::red_at_two,
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

/// The observer's reading of a `;`-separated names line the writer re-parts, the last name's grade written after its
/// comma in two italic pieces (vgtc, 2009.02373): the reading leaves the grade out of the name, so the re-part finds
/// each name in today's; the grade one `membership` contact. Red (106c's first build): "Ann Able; Bob Baker" one
/// creator. Repro sectioning-frontmatter/author_semicolon_names_written_grade_are_reparted.
#[test]
fn a_written_grade_leaves_the_readings_name_too() {
  if red_at_two(r#"writes a before=" " separator on Bob's creator the off reading leaves off"#) {
    return;
  }
  assert_eq!(
    clean_creators(
      repro!("author_semicolon_names_written_grade_are_reparted"),
      &[]
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> </creator> "#,
      r#"<creator role="author"> <personname>Bob Baker</personname> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      r#"<contact role="membership"><text font="italic" xml:id="id1">Senior Member</text>, "#,
      r#"<text font="italic" xml:id="id2">IEEE</text></contact> </creator>"#,
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

/// webofc's `Ann Able\inst{1}\fnsep\thanks{\email{…}} for CAPP/IBS` (1702.03664): the remark after the notes is her
/// collaboration contact, though it names no collaboration word. Red (int134b): personname "Ann Able for CAPP/IBS".
/// Repro class-bindings/webofc/webofc_remark_after_the_notes_is_a_contact.
#[test]
fn a_remark_after_the_notes_naming_whom_the_author_speaks_for_is_a_contact() {
  if red_at_two(r#"moves the affiliation after the email and collaboration contacts"#) {
    return;
  }
  assert_eq!(
    clean_creators(
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc_remark_after_the_notes_is_a_contact.tex"
      ),
      &[(
        "webofc.cls",
        include_str!("../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc.cls"),
      )],
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Korea</contact> "#,
      r#"<contact name="Email: " role="email">ann@a.edu</contact> "#,
      r#"<contact role="collaboration">for CAPP/IBS</contact> </creator>"#,
    )
  );
}

/// An author line opened by layout (`\vspace{-2em}\hspace{-25pt}` before the names, neurips 2002.03043, 1307.6590):
/// the kernel's split hands `\lx@add@author` a content too short to be a name line, which `name_line_span` must not
/// index — the A/B r106c's first build panicked there (Fatal). Converts clean, every creator as int134b reads them (the
/// run "Cat Cole Dan Dee" after `\and`, which the source separates by a line break alone, that reader's). Repro
/// sectioning-frontmatter/author_spacing_opened_name_lines_read_safely.
#[test]
fn an_author_line_opened_by_layout_is_read_safely() {
  if red_at_two(r#"drops the before=" " separator on Ann's creator"#) {
    return;
  }
  assert_eq!(
    clean_creators(repro!("author_spacing_opened_name_lines_read_safely"), &[]),
    concat!(
      r#"<creator before=" " role="author"> <personname>Ann Able<note mark="*" role="footnotemark" xml:id="footnotex1">"#,
      r#"<tags> <tag>*</tag> <tag role="refnum">*</tag> <tag role="typerefnum">footnote *</tag> </tags></note></personname> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole Dan Dee</personname> </creator>"#,
    )
  );
}

/// An IEEE name block whose names macros print, each with its written grade (1510.03162): `\lx@add@author` reads no
/// grade out of a macro's text, so the observer's reading gives each grade back to its name (writer/mod.rs
/// `give_back_grades`) and the re-part parts the four names as int135e. The reading route of the r2 fix. Red (106c
/// r1): one creator holding the four names and grades. Repro
/// sectioning-frontmatter/ieee_block_macro_names_with_grades_stay_apart.
#[test]
fn macro_printed_names_with_grades_stay_apart() {
  if red_at_two(r#"writes the names and grades as plain text, 9 false Warnings"#) {
    return;
  }
  assert_eq!(
    clean_creators(repro!("ieee_block_macro_names_with_grades_stay_apart"), &[]),
    concat!(
      r#"<creator role="author"> <personname>Ann Able, <text font="italic" xml:id="id1">Student Member, IEEE</text>"#,
      r#"</personname> </creator> "#,
      r#"<creator role="author"> <personname>Bob Baker, <text font="italic" xml:id="id2">Senior Member, IEEE</text>"#,
      r#"</personname> </creator> "#,
      r#"<creator role="author"> <personname>Cat Cole, <text font="italic" xml:id="id3">Member, IEEE</text>"#,
      r#"</personname> </creator> "#,
      r#"<creator role="author"> <personname>Dan Dee, <text font="italic" xml:id="id4">Senior Member, IEEE</text>"#,
      r#"</personname> <note class="ltx_note_frontmatter ltx_thanks_funding" role="thanks" xml:id="id5">Funded.</note> "#,
      r#"</creator>"#,
    )
  );
}

/// `\IEEEmembership{Member,~IEEE, Cat~Cole, Member,~IEEE}`, a grade, the next name and its grade in one membership
/// (2507.19818): the same give-back on a grade before a name, the class's membership route — three creators as
/// int135e, each `*` note its name's. Red (106c r1): "Bob Baker*Member, IEEE, Cat Cole, Member, IEEE" one creator, Ann
/// Able's note lost. Repro sectioning-frontmatter/ieee_membership_holding_a_name_stays_apart.
#[test]
fn a_membership_holding_the_next_name_stays_apart() {
  if red_at_two(r#"loses "IEEE Member", a content-check Warning"#) {
    return;
  }
  assert_eq!(
    clean_creators(repro!("ieee_membership_holding_a_name_stays_apart"), &[]),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id1">"#,
      r#"<sup xml:id="id1.1">*</sup>Equal contribution.</note> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker*Member, IEEE</personname> "#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id2">"#,
      r#"<sup xml:id="id2.1">*</sup>Equal contribution.</note> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole, Member, IEEE</personname> "#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_funding" role="thanks" xml:id="id3">Funded.</note> </creator>"#,
    )
  );
}

/// The writer route of the r2 fix: `\lx@add@author` takes a grade out only when the line before it prints none
/// (name_text.rs `holds_grade`) — here a grade before a later name inside a bold group the kernel's split does not
/// enter. Every grade stays, the two names part as int135e. Red (106c r1): one creator "Ann Able, Member, IEEE; Bob
/// Baker" and the contact "Fellow, IEEE". Repro sectioning-frontmatter/author_grade_before_a_later_name_stays_whole.
#[test]
fn a_grade_before_a_later_name_keeps_the_line_whole() {
  if red_at_two(
    r#"parts the bold name line, writing "Member, IEEE" as a membership contact (off keeps the line whole)"#,
  ) {
    return;
  }
  assert_eq!(
    clean_creators(repro!("author_grade_before_a_later_name_stays_whole"), &[]),
    concat!(
      r#"<creator role="author"> <personname><text font="bold" xml:id="id1">Ann Able, "#,
      r#"<text font="italic" xml:id="id1.1">Member, IEEE</text></text></personname> </creator> "#,
      r#"<creator role="author"> <personname><text font="bold" xml:id="id2">Bob Baker</text>, "#,
      r#"<text font="italic" xml:id="id3">Fellow, IEEE</text></personname> </creator>"#,
    )
  );
}

/// A remark after a name's mark is the collaboration the author speaks for only when its words name a group — acronyms,
/// slashed names, a team's word (`speaks_for_after_notes`): a funding line and a company stay in the personname as on
/// the base, the ATLAS Collaboration a contact. Review rev106c SHOULD-FIX. Red (106c r1): "for NSF Grant 123" and "for
/// ACME Corp" collaboration contacts. Repro sectioning-frontmatter/collab_for_remark_not_a_group.
#[test]
fn a_remark_naming_no_group_stays_the_names() {
  if red_at_two(r#"makes "for NSF Grant 123" a creator of its own"#) {
    return;
  }
  assert_eq!(
    clean_creators(repro!("collab_for_remark_not_a_group"), &[]),
    concat!(
      r#"<creator role="author"> <personname>Ann Able for NSF Grant 123<sup xml:id="id1">1</sup></personname> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> <personname>Cy Coe for ACME Corp<sup xml:id="id2">2</sup></personname> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> <personname>Di Dee<sup xml:id="id3">3</sup></personname> "#,
      r#"<contact role="collaboration">for the ATLAS Collaboration</contact> </creator>"#,
    )
  );
}
