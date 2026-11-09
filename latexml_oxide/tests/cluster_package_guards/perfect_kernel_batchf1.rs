//! Guards for F1 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, Route D, the flip's prerequisites; OXIDIZED_DESIGN_DIVERGENCES #837,
//! #980): positions per call, so a block past the 16,383 positions the field's stamps take is read with the emitter's switch on
//! (`a_block_past_the_stamps_positions_is_read`), and a creator call inside a block keeps its role (`a_creator_call_in_a_block_keeps_its_role`)
//! and its keyvals (`a_creator_calls_keyvals_are_written`); F1r2: a link printing a name is that name
//! (`a_link_printing_a_name_is_the_name`), and a grade is read however long its group (`a_grade_with_a_long_note_is_the_grade`).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::fm_shadow_json;
use regex::Regex;

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

/// The fixtures' generated names' suffix: the digits of `n`, each a letter `a`..`j` (their `\letters`).
fn letters(n: usize) -> String {
  n.to_string()
    .chars()
    .map(|d| char::from(b'a' + d.to_digit(10).map_or(0, |d| d as u8)))
    .collect()
}

/// `tex` converted under the census switch: the log, the XML and the shadow dump (`fm_shadow_json`).
fn run_with_shadow(tex: &str) -> (String, String, String) {
  convert_files_with_setup_then(
    tex,
    &[],
    Some("ar5iv.sty"),
    move || {},
    |_| fm_shadow_json(),
  )
}

/// Positions per call (`ledger::record_author_call`, the author field's 15 bits, `token::AUTHOR_PROV_POS_MASK`): a kernel `\author` of
/// 650 names, 16,899 tokens, past the 16,383 positions its `\lx@field@mark` takes (BESIII's 2204.09614, 18,395 tokens; 2404.06718,
/// 2410.11607), and 500 revtex `\author` calls of 16,500 tokens together (2607.15927, 2512.08803), are read under `=2`: every name its
/// creator, no Warning. Each is one distinct path: the kernel block whose mark ran out, and the binding calls past them together. With the
/// switch off both stay as the old reader cuts them, the blocks past the 14 bits the field's calls take together out of positions
/// (`ledger::out_of_positions`), the dump's stamp `overflow` beside its reason, as before positions were per call; under `=2` none is. On
/// int145a the kernel half reads one creator under `=2`; the revtex half's creators are the same there, its RED the Warning's count alone.
#[test]
fn a_block_past_the_stamps_positions_is_read() {
  let (log, xml) = run(repro!("emit_a_block_past_the_stamps_positions_is_read"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let names: Vec<Vec<String>> = std::iter::once(vec!["Annabelle Abernathy-a".to_string()])
    .chain((1..650).map(|n| vec![format!("Annabelle Abernathy-{}", letters(n))]))
    .collect();
  assert_eq!(creators(&xml), names);

  let (log, xml, json) = run_with_shadow(repro!(
    "emit_calls_past_the_stamps_positions_together_are_read"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // (each block's stamp and why it went unread, its reason when it did)
  let stamps = |json: &str| -> Vec<(String, Option<String>)> {
    let v: serde_json::Value = serde_json::from_str(json).expect("the shadow JSON");
    let entries = v["entries"].as_array().cloned().unwrap_or_default();
    entries
      .iter()
      .filter(|e| e["kind"] == "author")
      .map(|e| {
        (
          e["stamp"].as_str().unwrap_or_default().to_string(),
          e["reason"].as_str().map(str::to_string),
        )
      })
      .collect()
  };
  assert_eq!(stamps(&json), vec![("projected".to_string(), None); 500]);
  let authors: Vec<Vec<String>> = (0..500)
    .map(|n| {
      vec![
        format!("Annabelle Abernathy-Brandenburg B{}", letters(n)),
        "affiliation=Univ A, Town".to_string(),
      ]
    })
    .collect();
  assert_eq!(creators(&xml), authors);
}

/// A creator call a block's own expansion makes keeps its keyvals on the Name it prints (`field_notes::FieldCreator`,
/// `events::creator_roles`): PoS's `\speaker{Ann Able}` (`\lx@add@creator[role=speaker]`, 0910.5686, 1007.2355, 1111.2428, 1111.6944) is
/// `role=speaker` under `=1` and `=2` as with the switch off, written from the call's name and not the call (`writer::emit::event_run`:
/// the call written again added a second creator, a nameless one, holding Ann Able's institute), Bob Baker beside it `role=author`.
#[test]
fn a_creator_call_in_a_block_keeps_its_role() {
  let tex = repro!("emit_a_creator_call_in_a_block_keeps_its_role");
  let roles = |xml: &str| -> Vec<String> {
    Regex::new(r#"<creator\b[^>]*\brole="([^"]*)""#)
      .expect("regex")
      .captures_iter(xml)
      .map(|c| c[1].to_string())
      .collect()
  };
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(roles(&xml), ["speaker", "author"]);
  assert_eq!(creators(&xml), vec![
    vec!["Ann Able", "affiliation=Univ A, Town", "email=ann@a.edu"],
    vec!["Bob Baker", "affiliation=Univ B, City"]
  ]);
  assert_eq!(creators_xml(&xml).len(), 2);
}

/// A link that carries a mark and prints other text than its own target is read as that text, a Name carrying the link
/// (`resolver::lines::link_names`, `prints_its_target`): 1812.07638's 304 `\href{http://inspirehep.net/record/…}{A.~Cerri}$^{1}$`, its
/// span the link's call, written whole with its link, its institutes by its marks. A link printing its own target stays a contact
/// (`\href{mailto:ann@x.org}{ann@x.org}`, `\url{…}`), and so does one carrying no mark (`\href{…}{project page}`,
/// `perfect_kernel_batchfms2a1::sync_row_shapes_47_48_links`) and one to an ORCID record is its ORCID beside the name (the same pass). Under `=2` every creator is
/// the switch-off one, element for element; on F1 the names were url contacts, one creator's affiliation, and a nameless creator held
/// the institutes.
#[test]
fn a_link_printing_a_name_is_the_name() {
  let tex = repro!("emit_a_link_printing_a_name_is_the_name");
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators_xml(&xml), vec![
    "<creator role=\"author\">\n    <personname>Editors</personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname><ref class=\"ltx_href\" href=\"http://inspirehep.net/record/1\">A.\u{a0}Cerri</ref></personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Town</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname><ref class=\"ltx_href\" href=\"http://inspirehep.net/record/2\">T.\u{a0}\u{a0}Hurth</ref></personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, City<break/>E-mail: <ref class=\"ltx_href\" href=\"mailto:ann@x.org\">ann@x.org</ref>, <ref class=\"ltx_url\" font=\"typewriter\" href=\"http://x.org/~ann\">http://x.org/~ann</ref></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname><ref class=\"ltx_href\" href=\"http://inspirehep.net/record/3\">S.\u{a0}Malvezzi</ref></personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Town</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, City<break/>E-mail: <ref class=\"ltx_href\" href=\"mailto:ann@x.org\">ann@x.org</ref>, <ref class=\"ltx_url\" font=\"typewriter\" href=\"http://x.org/~ann\">http://x.org/~ann</ref></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, City<break/>E-mail: <ref class=\"ltx_href\" href=\"mailto:ann@x.org\">ann@x.org</ref>, <ref class=\"ltx_url\" font=\"typewriter\" href=\"http://x.org/~ann\">http://x.org/~ann</ref></contact>\n  </creator>"
  ]);
  let names: Vec<String> = creators(&xml).into_iter().map(|c| c[0].clone()).collect();
  assert_eq!(names, [
    "Editors",
    "A. Cerri",
    "T. Hurth",
    "S. Malvezzi",
    "Bob Baker"
  ]);
  let links =
    Regex::new(r#"<personname><ref class="ltx_href" href="http://inspirehep.net/record/(\d)">"#)
      .expect("regex");
  let linked: Vec<String> = links
    .captures_iter(&xml)
    .map(|c| c[1].to_string())
    .collect();
  assert_eq!(linked, ["1", "2", "3"]);
}

/// `\lx@add@author`'s membership readout (`author_block::name_text::written_membership_out`) reads a grade as a suffix of grade words,
/// its commas from the last leftward, stopping at the first tail holding another word: linear (the readout per comma was quadratic on
/// `=2`'s queued block of 418 names, 1507.02068) with no cap on what the grade's group holds. `{\itshape Fellow, IEEE\thanks{<300
/// characters>}}` is Ann Able's `membership` with the switch off, as int145a's, the thanks the document's pubnote there; F1's 256-token
/// cap read it into her name. Under `=2` the thanks is her note.
#[test]
fn a_grade_with_a_long_note_is_the_grade() {
  let tex = repro!("emit_a_grade_with_a_long_note_is_the_grade");
  let grades = [vec!["Ann Able", "membership=Fellow, IEEE"], vec![
    "Bob Baker",
    "membership=Member, IEEE",
  ]];
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), grades);
  let ann = &creators_xml(&xml)[0];
  assert!(ann.contains(r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">A. Able is with the Department"#), "{ann}");
}

/// A creator call inside a block carries its keyvals as tokens (the rehearsal's `\lx@fm@obs@creator` reads them as the API does,
/// `field_notes::FieldCreator`), and the emitter writes them on Perl's call `[role=author,#1]` (Base_Utility.pool.ltxml:586,
/// `writer::emit::creator_keys`): `\lx@add@creator[role=editor,name={Edited by},annotations={ed}]{Bob Baker}` (ams's `\contrib` gives
/// `name=`, icml's `\icmlauthor` `annotations=`) is `<creator name="Edited by" role="editor">` under `=2`; F1 kept the role alone. The
/// switch-off path is the kernel reader's, which cuts the keyvals at their commas — not this path.
#[test]
fn a_creator_calls_keyvals_are_written() {
  let (log, xml) = run(repro!("emit_a_creator_calls_keyvals_are_written"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = creators_xml(&xml);
  let squeezed: Vec<String> = xml
    .iter()
    .map(|c| c.split_whitespace().collect::<Vec<_>>().join(" "))
    .collect();
  assert_eq!(squeezed, [
    r#"<creator role="author"> <personname>Ann Able</personname> </creator>"#,
    r#"<creator name="Edited by" role="editor"> <personname>Bob Baker</personname> </creator>"#,
  ]);
}
