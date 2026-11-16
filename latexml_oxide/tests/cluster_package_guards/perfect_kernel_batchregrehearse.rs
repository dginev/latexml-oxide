//! Rows for batch REGREHEARSE: two regressions of LOSTF's rehearsal under the author markup (#866, OXIDIZED_DESIGN_DIVERGENCES
//! #941), each red on the cutoff 999af99497d. A title-code command that assigns besides printing (klmm.sty's `\instref`)
//! made no declaration of the block to replay apart, where it is undefined (2606.12100, 2606.12130, 2606.12144, 2606.22997,
//! 2606.26035: `Error:undefined:\instref`); OmniBus's `\orcid`, which the markup provides for a class defining its own only
//! inside its `\author`, is one ORCID call with its argument whole (2606.13132: `Error:undefined:\@undefined` and the iD
//! given as its address lost). Each row converts its repro once, in-process, and asserts its exact Error/Warning counts,
//! every creator's name and contacts, and the creators' elements.
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

const PRELOAD: &str = "ar5iv.sty";

const TITLE_CODE: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_code_command_assigning_is_no_declaration.tex"
);
const ORCID: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/omnibus_orcid_given_as_its_address_keeps_the_id.tex"
);

/// Each `<creator …>…</creator>` element of `xml`, in order.
fn creator_elements(xml: &str) -> Vec<&str> {
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let end = start + xml[start..].find("</creator>").expect("closed creator");
      &xml[start..end]
    })
    .collect()
}

/// The title-code command's assignments (`field_notes::keep_declared`): called from the block (`\instref{1}`, the witnesses'
/// shape) and through a paper's macro (`\myref{2}`); the variant calls it only through the macro, the path the first call's
/// Error, raised once, would hide. Both marks stay the names'.
#[test]
fn title_code_command_assigning_is_no_declaration() {
  let variant = TITLE_CODE.replace(r"Ann Able\instref{1}", r"Ann Able\myref{1}");
  assert_ne!(variant, TITLE_CODE);
  for tex in [TITLE_CODE, variant.as_str()] {
    let (log, xml) = convert_with(tex, Some(PRELOAD));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(
      creators(&xml),
      [vec!["Ann Able 1".to_string()], vec![
        "Bob Baker 2".to_string()
      ]],
      "{xml}"
    );
    let elements = creator_elements(&xml);
    assert!(
      elements[0].contains(r#"<personname>Ann Able<sup xml:id="id1">1</sup></personname>"#),
      "{xml}"
    );
    assert!(
      elements[1].contains(r#"<personname>Bob Baker<sup xml:id="id2">2</sup></personname>"#),
      "{xml}"
    );
  }
}

/// OmniBus's `\orcid` in the rehearsal (`\lx@author@orcid` bound as `\lx@fm@obs@orcid`): each creator keeps its iD, the one
/// given as its address (`orcid.org/0`) too.
#[test]
fn omnibus_orcid_given_as_its_address_keeps_the_id() {
  let (log, xml) = convert_with(ORCID, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    [vec!["Yu Kim".to_string(), "orcid=".to_string()], vec![
      "Ann Lee".to_string(),
      "orcid=".to_string()
    ]],
    "{xml}"
  );
  let elements = creator_elements(&xml);
  assert!(
    elements[0].contains(
      r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0" title="ORCID 0">"#
    ),
    "{xml}"
  );
  assert!(
    elements[1].contains(
      r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0001-2345-6789" title="ORCID 0000-0001-2345-6789">"#
    ),
    "{xml}"
  );
}
