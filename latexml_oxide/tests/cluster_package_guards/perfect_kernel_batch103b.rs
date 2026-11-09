//! Red/green guards for batch 103b: name tails left inside `personname` (census129 cluster 5). (C) a name list a
//! binding splits at its commas and "and" (`\lx@add@author@split`: llncs, aa, inst_support, sv_support, webofc, ptephy)
//! reads the collaboration it names beside its names, as the kernel's author line did (the old driver's `split_author_calls`, deleted in DEL2;
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

/// (O) revtex4-2's `\author{Ann Able \orcidA{}}`, `\orcidA` an `\href` to her ORCID record round a TikZ "iD": the
/// personname is the name, the link her `contact role="orcid"` with the kernel's ORCID link and logo; Bob Baker's link
/// to another address stays in his name and the ORCID link in his `\thanks` stays the note's text. Red (int130a):
/// `<personname>Ann Able <ref class="ltx_href" href="https://orcid.org/…"><picture …>`. Repro sectioning-frontmatter/
/// author_orcid_href_icon_is_the_orcid.
#[test]
fn an_orcid_link_in_a_name_is_the_authors_orcid() {
  let (log, _) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_orcid_href_icon_is_the_orcid.tex"
    ),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
}

/// Converts `repro` (with `files`) and returns its creators, asserting no Error or Warning.
fn clean_creators(repro: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = convert_files_with(repro, files, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators(&xml)
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
