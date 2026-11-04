//! Red/green guards for batch 103b: name tails left inside `personname` (census129 cluster 5). (C) a name list a
//! binding splits at its commas and "and" (`\lx@add@author@split`: llncs, aa, inst_support, sv_support, webofc, ptephy)
//! reads the collaboration it names beside its names, as the kernel's author line does (`split_author_calls`,
//! OXIDIZED_DESIGN_DIVERGENCES #946): webofc's `… on behalf of the KLOE-2 Collaboration` after the name's notes
//! (1502.05517, 1904.12569, 2411.14337, 1611.00816, 1512.03605). (O) an `\href` to an ORCID record in an author's
//! name that prints no word is the author's ORCID contact, its ink no part of the name
//! (`\lx@author@href`, OXIDIZED_DESIGN_DIVERGENCES #960): the `\orcidicon` template's TikZ "iD" (2507.12319,
//! 2503.04867, 2406.08258). Each guard converts its repro and asserts no Error or Warning and every creator whole.
use super::{
  convert_files_with,
  perfect_kernel_batch46::{error_count, warning_count},
};

const PRELOAD: &str = "ar5iv.sty";

/// Every creator of `xml`, whitespace squashed, the ORCID logo's SVG written `<SVG/>`.
fn creators(xml: &str) -> String {
  let mut xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  while let Some(start) = xml.find("<svg:svg") {
    let end = xml[start..].find("</svg:svg>").expect("a closed svg") + start + "</svg:svg>".len();
    xml.replace_range(start..end, "<SVG/>");
  }
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  xml[start..end].to_string()
}

/// (C) webofc's names with a collaboration remark after their marks and notes, after a `\ `, and before them: each
/// personname the name alone, the remark its `contact role="collaboration"`, its institutes and e-mail kept (the marks
/// after the remark the name's). Red (int130a): "Ann Able on behalf of the XYZ Collaboration", "Bob Baker (for the ABC
/// Collaboration)", "Cat Cole for the DEF Collaboration". Repro class-bindings/webofc/
/// webofc_collaboration_after_the_notes_is_a_contact.
#[test]
fn webofc_collaboration_after_the_notes_is_a_contact() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc_collaboration_after_the_notes_is_a_contact.tex"
    ),
    &[(
      "webofc.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc.cls"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A</contact> "#,
      r#"<contact name="Email: " role="email">ann@a.edu</contact> "#,
      r#"<contact role="collaboration">on behalf of the XYZ Collaboration</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact role="collaboration">(for the ABC Collaboration)</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ B</contact> "#,
      r#"<contact name="Email: " role="email">cat@c.edu</contact> "#,
      r#"<contact role="collaboration">for the DEF Collaboration</contact> </creator>"#,
    ),
    "{xml}"
  );
}

/// (O) revtex4-2's `\author{Ann Able \orcidA{}}`, `\orcidA` an `\href` to her ORCID record round a TikZ "iD": the
/// personname is the name, the link her `contact role="orcid"` with the kernel's ORCID link and logo; Bob Baker's link
/// to another address stays in his name and the ORCID link in his `\thanks` stays the note's text. Red (int130a):
/// `<personname>Ann Able <ref class="ltx_href" href="https://orcid.org/…"><picture …>`. Repro sectioning-frontmatter/
/// author_orcid_href_icon_is_the_orcid.
#[test]
fn an_orcid_link_in_a_name_is_the_authors_orcid() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_orcid_href_icon_is_the_orcid.tex"
    ),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0003-0320-5768" "#,
      r#"title="ORCID 0000-0003-0320-5768"><SVG/></ref></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname><ref class="ltx_href" href="https://bob.example.org">Bob Baker</ref></personname> "#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">See "#,
      r#"<ref class="ltx_href" href="https://orcid.org/0000-0002-1825-0097">his record</ref>.</note> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#,
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<svg:svg class="ltx_orcidlogo""#) && !xml.contains("<picture"),
    "{xml}"
  );
}
