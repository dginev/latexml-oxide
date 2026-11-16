//! Rows for batch REGREHEARSE: two regressions of LOSTF's rehearsal under the author markup (#866, OXIDIZED_DESIGN_DIVERGENCES
//! #941), each red on the cutoff 999af99497d. A title-code command that assigns besides printing (klmm.sty's `\instref`)
//! made no declaration of the block to replay apart, where it is undefined (2606.12100, 2606.12130, 2606.12144, 2606.22997,
//! 2606.26035: `Error:undefined:\instref`); OmniBus's `\orcid`, which the markup provides for a class defining its own only
//! inside its `\author`, is one ORCID call with its argument whole (2606.13132: `Error:undefined:\@undefined` and the iD
//! given as its address lost). Each row converts its repro once, in-process, and asserts its exact Error/Warning counts,
//! every creator's name and contacts, and the creators' elements whole. Round 2 (review revREGREHEARSE): a paper's macro
//! calling a title-code command declares everything but what the command's run assigns, replayed under the author markup
//! (B1: a name built at run time was lost; S2: a named one erred `undefined:\instref`); an ORCID given as its `http://`
//! address loses no word.
use latexml::util::test::convert_with;
use serde_json::json;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz1::{blocks, dump},
};

const PRELOAD: &str = "ar5iv.sty";

const TITLE_CODE: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_code_command_assigning_is_no_declaration.tex"
);
const ORCID: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/omnibus_orcid_given_as_its_address_keeps_the_id.tex"
);
const IN_A_MACRO: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_code_command_in_a_papers_macro_keeps_its_other_definitions.tex"
);

/// Each `<name …>…</name>` element of `xml`, whole, in order.
fn elements<'a>(xml: &'a str, name: &str) -> Vec<&'a str> {
  let (open, close) = (format!("<{name}"), format!("</{name}>"));
  xml
    .match_indices(&open)
    .filter(|(start, _)| matches!(xml.as_bytes().get(start + open.len()), Some(b' ' | b'>')))
    .map(|(start, _)| {
      let end = start + xml[start..].find(&close).expect("closed") + close.len();
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
    assert_eq!(
      elements(&xml, "personname"),
      [
        r#"<personname>Ann Able<sup xml:id="id1">1</sup></personname>"#,
        r#"<personname>Bob Baker<sup xml:id="id2">2</sup></personname>"#
      ],
      "{xml}"
    );
  }
}

/// A paper's macro calling a title-code command (review revREGREHEARSE B1, S2): what the command's run assigns is the title
/// code's, the run surviving the mouth its `\textsuperscript` digests its argument in (`field_notes::close_runs`) — no
/// `\current@has@affil` declared —, and every other definition of the macro is the block's, a name built at run time
/// (`\setaff`'s `\csname aff1\endcsname`) or written (`\setcity`'s `\cityA`), each span the macro's call; the replay of
/// those calls runs under the author markup (`emit::declarations`), so neither is lost nor errs `undefined:\instref`.
#[test]
fn title_code_command_in_a_papers_macro_keeps_its_other_definitions() {
  let (xml, dump) = dump(IN_A_MACRO);
  assert_eq!(
    blocks(&dump)[0]["declarations"],
    json!([
      {"kind": "Declaration", "span": [8, 22], "label": "\\aff1", "role": "global"},
      {"kind": "Declaration", "span": [33, 37], "label": "\\cityA", "role": "global"},
    ]),
    "{dump}"
  );
  assert_eq!(
    creators(&xml),
    [vec!["Ann Able 1".to_string()], vec![
      "Bob Baker 2".to_string()
    ]],
    "{xml}"
  );
  assert_eq!(
    elements(&xml, "personname"),
    [
      r#"<personname>Ann Able<sup xml:id="id1">1</sup></personname>"#,
      r#"<personname>Bob Baker<sup xml:id="id2">2</sup></personname>"#
    ],
    "{xml}"
  );
  assert_eq!(
    elements(&xml, "p"),
    [r#"<p xml:id="p1.1">Body: Inst One; City A.</p>"#],
    "{xml}"
  );
}

/// OmniBus's `\orcid` in the rehearsal (`\lx@author@orcid` bound as `\lx@fm@obs@orcid`): each creator keeps its iD, the one
/// given as its address (`orcid.org/0`) too, and one given as its `http://` address loses no word to the content check
/// (`placed::without_scheme`; review revREGREHEARSE).
#[test]
fn omnibus_orcid_given_as_its_address_keeps_the_id() {
  let (log, xml) = convert_with(ORCID, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    [
      vec!["Yu Kim".to_string(), "orcid=".to_string()],
      vec!["Ann Lee".to_string(), "orcid=".to_string()],
      vec!["Cy Ng".to_string(), "orcid=".to_string()]
    ],
    "{xml}"
  );
  // (each contact whole, its logo's SVG, the same in each, cut to its opening tag)
  let contacts: Vec<String> = elements(&xml, "contact")
    .into_iter()
    .map(|contact| {
      let start = contact.find("<svg:svg ").expect("the logo");
      let end = contact.find("</svg:svg>").expect("the logo's end") + "</svg:svg>".len();
      let open = &contact[start..start + contact[start..].find('>').expect("the logo's tag") + 1];
      format!("{}{open}{}", &contact[..start], &contact[end..])
    })
    .collect();
  let logo =
    r#"<svg:svg class="ltx_orcidlogo" height="1em" version="1.1" viewBox="0 0 72 72" width="1em">"#;
  assert_eq!(
    contacts,
    [
      format!(
        r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0" title="ORCID 0">{logo}</ref></contact>"#
      ),
      format!(
        r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0001-2345-6789" title="ORCID 0000-0001-2345-6789">{logo}</ref></contact>"#
      ),
      format!(
        r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" title="ORCID 0000-0002-1825-0097">{logo}</ref></contact>"#
      ),
    ],
    "{xml}"
  );
}
