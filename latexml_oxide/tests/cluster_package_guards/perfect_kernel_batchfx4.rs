//! Guards for FX4, an ORCID link printing the author's name under `LATEXML_FM_EMIT=2` (OXIDIZED_DESIGN_DIVERGENCES #980): the
//! name written once (`an_orcid_link_printing_the_name_is_written_once`).

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
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

/// An ORCID link whose printed text is the name (`link_names` reads it as the name and its ORCID) is written once: the name's
/// run prints the link, so the personname's `ltx:ref` to orcid.org is the creator's ORCID, as off and Perl hold it, and no
/// `contact role="orcid"` writes the link and the name again. Two shapes: authblk's whole call, an icon's box and a `\thanks`
/// inside the link (`\href{https://orcid.org/…}{\usebox{\orcid}\hspace{1mm}Luca Fehlings\thanks{…}}`, 2410.07791, 2507.22485,
/// 2602.05817), and links between a names line's separators with a mark after each (`\href{…}{E. L. Gagnon}$^{*,1,2}$`,
/// 2312.16289). On FX4base `=2` each name was held twice, with the content check's "more often" Warning; every creator is now the
/// switch-off one, element for element. Distinct from `perfect_kernel_batchlo1::a_link_filling_a_name_slot_is_the_name` (a
/// link that is no ORCID is the name alone, no contact made).
#[test]
fn an_orcid_link_printing_the_name_is_written_once() {
  for tex in [
    repro!("authblk_orcid_link_wrapping_the_name_written_once"),
    repro!("orcid_link_names_in_one_author_line_written_once"),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let found: Vec<String> = creators(&xml).into_iter().map(|c| c[0].clone()).collect();
    assert_eq!(found, ["Ann Able", "Bob Baker"]);
    for (creator, id) in creators_xml(&xml)
      .iter()
      .zip(["0000-0001-0000-0001", "0000-0002-0000-0002"])
    {
      assert!(
        creator.contains(&format!(
          r#"<personname><ref class="ltx_href" href="https://orcid.org/{id}">"#
        )),
        "{creator}"
      );
      assert!(!creator.contains(r#"role="orcid""#), "{creator}");
      assert_eq!(
        creator.matches("Able").count() + creator.matches("Baker").count(),
        1,
        "{creator}"
      );
    }
  }
}
