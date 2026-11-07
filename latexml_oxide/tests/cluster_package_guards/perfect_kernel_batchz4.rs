//! Guards for Z4 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, Route D; OXIDIZED_DESIGN_DIVERGENCES #979): the emitter behind `LATEXML_FM_EMIT`, each
//! fixture converted with the switch on and off in the conversion's thread. On a paper where the reading and the output agree the creators' XML is the
//! same, ids masked (`the_emit_is_the_old_markup`, one row per markup class); each other guard is a code path of its own: `off_is_untouched` the switch,
//! `an_environment_by_its_frames` the digested extents, `a_raw_environment_on_a_name_is_whole` what no frame says, `the_whole_block_queue_still_reads` `=2`.

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};
use regex::Regex;

use super::perfect_kernel_batch84b::creators;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Convert `tex` under `emit`: the log and the XML.
fn run(tex: &str, emit: u8) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    &[],
    Some("ar5iv.sty"),
    move || {
      set_fm_shadow_override(Some(true));
      set_fm_emit_override(Some(emit));
    },
    |_| (),
  );
  (log, xml)
}

/// The creators of `xml`, whole, ids and whitespace between elements masked.
fn creators_xml(xml: &str) -> Vec<String> {
  let creator = Regex::new(r"(?s)<creator\b.*?</creator>").expect("regex");
  let ids = Regex::new(r#"\s(xml:id|about|labels|idref|fragid|xoref|xref|resource)="[^"]*""#)
    .expect("regex");
  creator
    .find_iter(xml)
    .map(|found| ids.replace_all(found.as_str(), "").into_owned())
    .collect()
}

/// `tex` emitted, clean, with its creators' XML what the old engine's is, and `holds` in the emitted XML.
fn same_markup(tex: &str, holds: &[&str]) -> Vec<String> {
  let (off_log, off) = run(tex, 0);
  let (log, on) = run(tex, 1);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let emitted = creators_xml(&on);
  assert_eq!(emitted, creators_xml(&off));
  for fragment in holds {
    assert!(
      emitted.iter().any(|creator| creator.contains(fragment)),
      "{fragment}: {emitted:#?}"
    );
  }
  emitted
}

/// The markup classes of S0's report, each a shape the plain-text items lost: marks on names (kept by a per-author class, read by a block), an entry's font, a formula, a
/// name's own tokens against a wrapper's cover, each call's place on its author, a call's own address line with its name.
#[test]
fn the_emit_is_the_old_markup() {
  same_markup(
    repro!("emit_marks_a_per_author_class_keeps_on_its_names"),
    &[
      r#"<personname>Ann Able<sup><text font="italic">1</text></sup></personname>"#,
      "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>",
    ],
  );
  same_markup(repro!("emit_marks_a_block_answers_or_shows"), &[
    "<personname>Ann Able</personname>",
    "<personname>Bob Baker<sup>∗</sup></personname>",
    "<personname>Cat Cole<sup>3</sup></personname>",
  ]);
  same_markup(repro!("emit_fonts_a_group_before_a_mark"), &[
    r#"<text fontsize="90%">Univ A, Town</text>"#,
  ]);
  same_markup(repro!("emit_math_on_names_and_entries"), &[
    r#"<Math mode="inline" tex="\alpha""#,
    r#"Institute of <Math mode="inline" tex="\beta""#,
  ]);
  same_markup(repro!("emit_names_cut_by_their_own_tokens"), &[
    "<personname>Ann Živković</personname>",
  ]);
  same_markup(repro!("emit_order_each_call_its_place_on_its_author"), &[]);
  same_markup(repro!("emit_entry_address_a_line_of_its_call"), &[
    r#"role="affiliation">Univ A<break/>Town</contact>"#,
    "<contact name=\"Email:\u{a0}\" role=\"email\">ann@a.edu</contact>",
  ]);
}

/// Each source call its own item, in order on each author: a shared entry at the place its call has there, not its first holder's (0704.3133, 1202.0103).
#[test]
fn each_call_keeps_its_place_on_its_author() {
  let (log, xml) = run(repro!("emit_order_each_call_its_place_on_its_author"), 1);
  assert_eq!(error_count(&log), 0, "{log}");
  let expected: [&[&str]; 3] = [
    &["Ann Able", "affiliation=Univ B", "affiliation=Univ A"],
    &["Bob Baker", "email=bob@b.edu", "affiliation=Univ A"],
    &[
      "Cat Cole",
      "email=cat@c.edu",
      "affiliation=Univ C",
      "altaffiliation=Univ D",
    ],
  ];
  assert_eq!(
    creators(&xml),
    expected
      .iter()
      .map(|c| c.iter().map(ToString::to_string).collect::<Vec<_>>())
      .collect::<Vec<_>>()
  );
}

/// A `\footnote` and a `\thanks` on a name make what they make in an author's content, told apart by the rehearsal's digested fact (0707.1799).
#[test]
fn a_note_keeps_its_kind() {
  let (log, xml) = run(repro!("emit_notes_a_footnote_and_a_thanks_their_kinds"), 1);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let emitted = creators_xml(&xml);
  assert!(
    emitted[0].contains("<contact name=\"Note:\u{a0}\" role=\"note\">A footnote of Ann.</contact>"),
    "{emitted:#?}"
  );
  assert!(
    emitted[1].contains(r#"role="thanks">Funded by the NSF.</note>"#),
    "{emitted:#?}"
  );
}

/// One address of a list, each its own contact, in the fonts it prints in: the size too (1711.05466, 2210.05039; the old writer's copy lost the size).
#[test]
fn an_address_of_a_list_keeps_its_fonts() {
  let (log, xml) = run(repro!("emit_addresses_in_their_fonts_one_contact_each"), 1);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let emitted = creators_xml(&xml);
  assert!(emitted[0].contains("<contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" fontsize=\"90%\">ann@a.edu</text></contact>"), "{emitted:#?}");
  assert!(
    emitted[1].contains(r#"<text font="typewriter" fontsize="90%">bob@a.edu</text>"#),
    "{emitted:#?}"
  );
}

/// An ORCID written on a name stays in it, logo and all, on every flush; no contact of it (2208.05251, 2209.13190).
#[test]
fn an_orcid_stays_on_its_name() {
  let (log, xml) = run(repro!("emit_orcid_stays_on_the_name"), 1);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let emitted = creators_xml(&xml);
  assert_eq!(emitted.len(), 1, "{emitted:#?}");
  assert!(
    emitted[0].contains(r#"<ref class="ltx_orcid""#),
    "{emitted:#?}"
  );
  assert!(!emitted[0].contains(r#"role="orcid""#), "{emitted:#?}");
  assert!(xml.contains("xmlns:svg"), "{xml}");
}

/// A macro's `\\begin{tikzpicture}…\\end{tikzpicture}`, all at its call's position, closes once: the address holding it keeps its
/// picture; an ORCID a mark's formula holds is a contact of its id, not of the formula and its mark (2210.03748, 2206.03602,
/// 2409.13798). Distinct from `an_environment_by_its_frames`: the steps of one position, a `\\begin` heard and the frames both.
#[test]
fn a_macros_environment_closes_once() {
  let (log, xml) = run(repro!("emit_a_macros_environment_closes_once"), 1);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let expected: Vec<Vec<String>> = [
    [
      "Ann Able",
      "orcid=0000-0001-8855-3635",
      "affiliation=Univ A",
    ],
    [
      "Bob Bell",
      "orcid=0000-0003-4102-380X",
      "affiliation=Univ B",
    ],
  ]
  .iter()
  .map(|row| row.iter().map(ToString::to_string).collect())
  .collect();
  assert_eq!(creators(&xml), expected, "{xml}");
  let emitted = creators_xml(&xml);
  let picture = Regex::new(r#"role="affiliation">Univ A\s*<picture"#).expect("regex");
  assert!(picture.is_match(&emitted[0]), "{emitted:#?}");
  assert!(emitted.iter().all(|c| !c.contains("<Math")), "{emitted:#?}");
}

/// A legend an API call holds is its carriers' note with its mark, on every author showing it (2607.02294, 1611.06354).
#[test]
fn a_legend_is_its_carriers_note() {
  let (log, xml) = run(repro!("emit_legend_a_call_prints_its_mark"), 1);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let emitted = creators_xml(&xml);
  assert!(
    emitted[0].contains(r#"mark="∗" role="thanks""#),
    "{emitted:#?}"
  );
  assert!(!emitted[1].contains("contributed equally"), "{emitted:#?}");
}

/// A second flush emits what is new only, and an emitted paper's second `\author` replaces as Perl's does (S0's `call: another flush's`).
#[test]
fn a_repeat_flush_emits_what_is_new() {
  let (log, xml) = run(repro!("emit_flush_again_keeps_what_it_emitted"), 1);
  assert_eq!(error_count(&log), 0, "{log}");
  let names: Vec<String> = creators(&xml).iter().map(|c| c[0].clone()).collect();
  assert_eq!(
    names.iter().filter(|name| *name == "Ann Able").count(),
    1,
    "{xml}"
  );
}

/// The switch off leaves the writer's path: the emitter's macros are not reached, and what the switch reads is `0`.
#[test]
fn off_is_untouched() {
  let (log, xml) = run(repro!("emit_marks_a_block_answers_or_shows"), 0);
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(!log.contains("lx@frontmatter@item"), "{log}");
  assert_eq!(creators(&xml).len(), 3, "{xml}");
}

/// `=2`, one whole-block entry per call: it reads and flushes clean, the emitter taking the entry (the next batch makes it reach `=1`).
#[test]
fn the_whole_block_queue_still_reads() {
  let (log, xml) = run(repro!("emit_marks_a_block_answers_or_shows"), 2);
  assert_eq!(error_count(&log), 0, "{log}");
  let names: Vec<String> = creators(&xml).iter().map(|c| c[0].clone()).collect();
  assert_eq!(names, ["Ann Able", "Bob Baker ∗", "Cat Cole 3"]);
}

/// A cut holds its environments as their meanings and the digestion's frames say them, whatever spells them: a macro pair and `\\let` copies open and close one, a `\\begin`
/// under `\\detokenize` or `\\ifx` opens none (the writer's cut, on the base's own path: `writer::Field::environments`; review revZ3r2).
#[test]
fn an_environment_by_its_frames() {
  let template = repro!("affiliation_cut_environments_by_spelling_red");
  let shape = "\\bx a&b\\\\ c&d\\ex";
  assert!(template.contains(shape));
  for (row, text) in [
    (shape, "1 Univ X a b c d"),
    (
      "\\bb{tabular}{cc}a&b\\\\ c&d\\ee{tabular}",
      "1 Univ X a b c d",
    ),
    ("\\detokenize{\\begin} X", "1 Univ \u{201c}begin X"),
    ("\\detokenize{\\end} X", "1 Univ \u{201c}end X"),
    ("\\ifx\\zz\\end Q\\else X\\fi", "1 Univ X"),
  ] {
    let template = if row.contains("&") {
      template.to_string()
    } else {
      template.replace("Univ X ", "Univ ")
    };
    let (log, xml) = run(&template.replace(shape, row), 0);
    assert_eq!(error_count(&log), 0, "{row}: {log}");
    let expected = vec![vec![
      "Ann Able".to_string(),
      format!("affiliation={text}"),
      "affiliation=2 Univ Y".to_string(),
    ]];
    assert_eq!(creators(&xml), expected, "{row}");
  }
}

/// A raw environment on a name is closed by its `\\end`, whose meaning says it: no frame binds `current_environment` for `{CJK*}` (2609.05616).
#[test]
fn a_raw_environment_on_a_name_is_whole() {
  let (log, xml) = run(repro!("emit_a_raw_environment_on_a_name"), 1);
  assert_eq!(error_count(&log), 0, "{log}");
  let expected = vec![vec![
    "Ann Able (xyz)".to_string(),
    "affiliation=Univ A".to_string(),
  ]];
  assert_eq!(creators(&xml), expected, "{xml}");
}
