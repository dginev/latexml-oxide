//! Red/green guards for batch 103b: name tails left inside `personname` (census129 cluster 5). (C) a name list a
//! binding splits at its commas and "and" (`\lx@add@author@split`: llncs, aa, inst_support, sv_support, webofc, ptephy)
//! reads the collaboration it names beside its names, as the kernel's author line does (`split_author_calls`,
//! OXIDIZED_DESIGN_DIVERGENCES #946): webofc's `… on behalf of the KLOE-2 Collaboration` after the name's notes
//! (1502.05517, 1904.12569, 2411.14337, 1611.00816, 1512.03605). (O) an `\href` to an ORCID record in an author's
//! name that prints no word is the author's ORCID contact, its ink no part of the name
//! (`\lx@author@href`, OXIDIZED_DESIGN_DIVERGENCES #960): the `\orcidicon` template's TikZ "iD" (2507.12319,
//! 2503.04867, 2406.08258). Round 2 (review rev103b): an icon glued after a name's comma marks is that name's
//! (2605.03577), a link printing letters or digits stays printed, and a creator holding one iD twice keeps one contact
//! (2501.02380). Each guard converts its repro and asserts no Error or Warning and every creator whole.
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

/// Converts `repro` (with `files`) and returns its creators, asserting no Error or Warning.
fn clean_creators(repro: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = convert_files_with(repro, files, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators(&xml)
}

/// (O, B1) an ORCID icon link glued after the marks (and the note) that follow a name's comma is that name's, as its
/// marks are: `Ann Able,$^{1}$\thanks{…}\orcidA⏎Bob Baker,$^{2}$\orcidB⏎and Carl Cole$^{1}$\orcidC` gives each name its
/// own iD, in the kernel's author line (article) and mnras's (the `\newauthor` between). Distinct from
/// `an_orcid_link_in_a_name_is_the_authors_orcid`: the split moves the icon, not the contact reading. Red (103b): Ann
/// Able no ORCID, Bob Baker hers and his own (2605.03577: each author's iD on the next). Repros sectioning-frontmatter/
/// author_orcid_icon_after_comma_marks_is_the_names, class-bindings/mnras_orcid_icon_after_comma_marks_is_the_names.
#[test]
fn an_orcid_icon_after_a_names_comma_is_its_own() {
  let orcid = |id: &str| {
    format!(
      r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/{id}" title="ORCID {id}"><SVG/></ref></contact> "#
    )
  };
  let names = |note: &str| {
    [
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      note,
      &orcid("0000-0002-1825-0097"),
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      &orcid("0000-0001-5109-3700"),
      r#"<contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Carl Cole</personname> "#,
      &orcid("0000-0002-1694-233X"),
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator>"#,
    ]
    .concat()
  };
  assert_eq!(
    clean_creators(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_orcid_icon_after_comma_marks_is_the_names.tex"
      ),
      &[]
    ),
    names(
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">E-mail: ann@a.edu</note> "#
    ),
    "article"
  );
  assert_eq!(
    clean_creators(
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/mnras_orcid_icon_after_comma_marks_is_the_names.tex"
      ),
      &[]
    ),
    names(""),
    "mnras"
  );
}

/// (O, S1) an ORCID link in a name is the author's contact only when its ink prints no letter or digit
/// (`ink_prints_text`): two icons are two contacts in order; a linked name (`\href{https://orcid.org/…}{Bob Baker}`)
/// stays the name; the iD printed as its digits, with or without the `X` checksum, stays printed, as pdflatex prints
/// it. Distinct from `an_orcid_link_in_a_name_is_the_authors_orcid` (an icon read from a picture): the ink read from
/// its characters. Red (103b): Carl Cole's digits an ORCID contact and logo, Dan Dorn's kept. Repro
/// sectioning-frontmatter/author_orcid_href_printing_text_stays.
#[test]
fn an_orcid_link_printing_text_stays_printed() {
  assert_eq!(
    clean_creators(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_orcid_href_printing_text_stays.tex"
      ),
      &[]
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" "#,
      r#"title="ORCID 0000-0002-1825-0097"><SVG/></ref></contact> "#,
      r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0001-5109-3700" "#,
      r#"title="ORCID 0000-0001-5109-3700"><SVG/></ref></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname><ref class="ltx_href" href="https://orcid.org/0000-0003-0320-5768">Bob Baker</ref></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Carl Cole "#,
      r#"<ref class="ltx_href" href="https://orcid.org/0000-0002-1825-0097">0000-0002-1825-0097</ref></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Dan Dorn "#,
      r#"<ref class="ltx_href" href="https://orcid.org/0000-0002-1694-233X">0000-0002-1694-233X</ref></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator>"#,
    )
  );
}

/// (O, S2) acmart's `\orcid{X}` beside the name's icon link to X (`\orcidicon{X}`, a macro of one argument) is one
/// contact, the first kept (`drop_repeated_orcids`); a different iD in each is two. Distinct: the writer's pass over
/// the placed contacts, not the reading. Red (103b): Ann Able's iD twice (2501.02380: each author's). Repro
/// class-bindings/acmart_orcid_and_its_icon_link_one_contact.
#[test]
fn a_repeated_orcid_is_one_contact() {
  assert_eq!(
    clean_creators(
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/acmart_orcid_and_its_icon_link_one_contact.tex"
      ),
      &[]
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" "#,
      r#"title="ORCID 0000-0002-1825-0097"><SVG/></ref></contact> "#,
      r#"<contact name="email: " role="email">ann@a.edu</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id1">"#,
      r#"Univ A</text>, <text class="ltx_affiliation_country" xml:id="id2">USA</text></contact> </creator> "#,
      r#"<creator before=" and " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0001-5109-3700" "#,
      r#"title="ORCID 0000-0001-5109-3700"><SVG/></ref></contact> "#,
      r#"<contact name="email: " role="email">bob@b.edu</contact> "#,
      r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1694-233X" "#,
      r#"title="ORCID 0000-0002-1694-233X"><SVG/></ref></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id3">"#,
      r#"Univ B</text>, <text class="ltx_affiliation_country" xml:id="id4">USA</text></contact> </creator>"#,
    )
  );
}
