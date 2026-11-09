//! Guards for Z1 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md` "Gaps in the event model", 1, 2, 6, 8, 9): what the
//! observer's events gain, read from the digested list and its spans, for the old engine's removal. Nothing reads these
//! fields but these guards; the reading and the writer's output stay as they were (the guards compare the whole events a
//! dump holds; `ab_audit fmshadow --re-resolve`'s test reads them back). Each guard is a different code path:
//! `marks_say_…` the walker on a stamped block, `a_call_marks_…` on an API call's content, which has no positions;
//! `a_contact_…` the ledger's unprinted call; `a_definition_…` the stomach's note of a boxless assignment,
//! `a_declarations_span_…` where that note's span starts and what it may not hold; the two `held` guards the document's own
//! error and an error of the rehearsal alone.

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use serde_json::{Value, json};

use super::perfect_kernel_batcht1::red_at_two;

/// Convert `tex` under the shadow switch: the log, the XML and the dump.
pub(super) fn run(tex: &str) -> (String, String, Value) {
  let (log, xml, json) = convert_files_with_setup_then(
    tex,
    &[],
    Some("ar5iv.sty"),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  (log, xml, serde_json::from_str(&json).expect("the dump"))
}

/// [`run`], asserting a clean conversion.
pub(super) fn dump(tex: &str) -> (String, Value) {
  let (log, xml, dump) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  (xml, dump)
}

/// The author blocks of a dump.
pub(super) fn blocks(dump: &Value) -> Vec<&Value> {
  dump["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .filter(|e| e["kind"] == "author")
    .collect()
}

/// A list's events of `kind`.
pub(super) fn of_kind(events: &Value, kind: &str) -> Vec<Value> {
  events
    .as_array()
    .expect("events")
    .iter()
    .filter(|e| e["kind"] == kind)
    .cloned()
    .collect()
}

/// A whole Mark made by `role`, printing `label`: its span, and what encloses it, `None` for nothing.
fn mark(
  label: &str,
  role: &str,
  span: (u16, u16),
  formula: Option<((u16, u16), Option<&str>, bool)>,
) -> Value {
  let mut mark = json!({
    "kind": "Mark", "span": [span.0, span.1], "text": label, "label": label, "role": role, "line_first": false,
    "symbol": label == "†",
  });
  if let Some((span, base, ends)) = formula {
    mark["formula_span"] = json!([span.0, span.1]);
    mark["ends_formula"] = json!(ends);
    if let Some(base) = base {
      mark["base"] = json!(base);
    }
  }
  mark
}

/// Gaps 1, 2 on a block: the formula's span is the whole `$…$`, its opening shift too (a Mark's own span the script
/// alone), a floating script and a posted one are told apart by what prints before them, and a prescript `${}^{4}$He` reads
/// as a mark does — no letter or digit after it in its formula, the fail-safe's input — where `${}^{5}\mathrm{He}$` has
/// one. A font switch, a space and an empty group are no base (2105.07958, 0804.4669); `$^{6,}$$^{7}$` are two formulas; a
/// glyph's mark in a formula reads as a script's. A mark at the block's top level has none of it; one in a group or a box
/// has that span and no base.
#[test]
fn marks_say_their_base_and_their_formula() {
  let (_, dump) = dump(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/event_mark_base_and_formula.tex"
  ));
  let floating = Some("floating");
  assert_eq!(of_kind(&blocks(&dump)[0]["events"], "Mark"), vec![
    mark("1", "math", (9, 13), Some(((8, 14), floating, true))),
    mark("2", "math", (27, 31), Some(((25, 32), floating, true))),
    mark(
      "3",
      "math",
      (46, 50),
      Some(((38, 51), Some("posted"), true))
    ),
    mark("4", "math", (64, 68), Some(((61, 69), floating, true))),
    mark("5", "math", (83, 87), Some(((80, 93), floating, false))),
    mark("a", "math", (104, 108), Some(((102, 109), floating, true))),
    mark("b", "math", (122, 126), Some(((121, 127), floating, true))),
    mark("6,", "math", (139, 144), Some(((138, 145), floating, true))),
    mark("7", "math", (146, 150), Some(((145, 151), floating, true))),
    mark("8", "math", (162, 166), Some(((161, 168), floating, true))),
    mark("†", "text", (166, 167), Some(((161, 168), floating, true))),
    mark("9", "sup", (178, 182), None),
    mark("10", "sup", (193, 198), Some(((192, 199), None, true))),
    mark("11", "sup", (209, 217), Some(((209, 217), None, true))),
    mark("†", "text", (228, 229), Some(((227, 230), floating, true))),
    mark("†", "text", (241, 242), Some(((240, 244), floating, false))),
  ]);
}

/// Gaps 1, 2 on an API call's content, which has no positions (`events` of a rehearsal of unstamped tokens): each Mark says
/// its base and what prints after it, its spans `null` — a font-switched and a spaced mark as their witnesses' calls
/// (2105.07958, 0804.4669), and an isotope leading an entry the writer cuts, with a letter after it.
#[test]
fn a_call_marks_say_their_base_without_positions() {
  let (_, dump) = dump(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/event_mark_in_entry_without_positions.tex"
  ));
  let marks: Vec<Value> = dump["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .filter(|e| e["kind"] == "api" && e["tag"] == "ltx:contact")
    .flat_map(|e| of_kind(&e["events"], "Mark"))
    .collect();
  let entry = |label: &str, ends: bool| {
    json!({
      "kind": "Mark", "span": null, "text": label, "label": label, "role": "math", "line_first": true, "symbol": false,
      "formula_span": null, "base": "floating", "ends_formula": ends,
    })
  };
  assert_eq!(marks, vec![
    entry("1", true),
    entry("2", true),
    entry("3", false)
  ]);
}

/// Gap 6: AASTeX 7's `\email` without `[show]` prints nothing and records an `ltx:rdf` (OD #479; 2506.18556, 2507.12222): a
/// Contact in the ledger's `unprinted`, its `text` empty, which the XML's `rdf` says too, on the author before it. Its
/// printing twin, `\email[show]`, is the API call it was, no unprinted one. Nothing reads it: the resolved creators hold
/// the printing address alone.
#[test]
fn a_contact_that_prints_nothing_says_so() {
  let (xml, dump) = dump(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/event_contact_unprinted.tex"
  ));
  assert!(
    xml.contains(r#"<rdf content="bob@b.org" property="schema:email"/>"#)
      && !xml.contains(r#"content="ann@a.org""#),
    "{xml}"
  );
  let entries = dump["entries"].as_array().expect("entries").len();
  assert_eq!(
    dump["unprinted"],
    json!([{
      "at": entries,
      "after_block": blocks(&dump)[1]["index"],
      "inside": false,
      "attrs": "property=schema:email,content=bob@b.org",
      "event": {"kind": "Contact", "span": null, "role": "schema:email", "unprinted": "bob@b.org"},
    }])
  );
  let contacts: Vec<&str> = dump["resolved"]["creators"]
    .as_array()
    .expect("creators")
    .iter()
    .flat_map(|c| c["contacts"].as_array().expect("contacts"))
    .filter_map(|c| c["text"].as_str())
    .collect();
  assert_eq!(contacts, vec!["ann@a.org"]);
}

/// Gap 8: a definition makes no box, so it stood in no event. Each is now a Declaration beside the block's events, with
/// its call's span and definees: a `\newcommand`, a `\def` in a group (`nest`), a paper's macro whose expansion defines
/// (its call's span), a `\global` one. The macro printing ink (`\printer`) has none, its box's span holding the call,
/// and a block with none has no list. A definition parts no name run, and each still defines where it stood: the names
/// and the note read the text they define.
#[test]
fn a_definition_in_the_block_is_an_event_with_its_span() {
  if red_at_two("loses \"BakerB Dept Q\" and writes \"Univ A\" as plain text, 3 Warnings") {
    return;
  }
  let (xml, dump) = dump(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/event_declaration_in_block.tex"
  ));
  let block = blocks(&dump)[0];
  assert_eq!(
    block["declarations"],
    json!([
      {"kind": "Declaration", "span": [0, 12], "label": "\\univ"},
      {"kind": "Declaration", "span": [27, 32], "label": "\\b", "nest": 1},
      {"kind": "Declaration", "span": [45, 46], "label": "\\dept"},
      {"kind": "Declaration", "span": [67, 73], "label": "\\q", "role": "global"},
    ])
  );
  assert!(of_kind(&block["events"], "Declaration").is_empty());
  let names: Vec<Value> = of_kind(&block["events"], "Name")
    .iter()
    .map(|name| name["text"].clone())
    .collect();
  assert_eq!(names, vec![
    "Ann Able",
    "Bob BakerB",
    "Cat Cole Dept",
    "Dan Dow Z",
    "Eve Ely Q"
  ]);
  assert!(xml.contains("Univ A"), "{xml}");
  // (its control twin: the other repro's blocks, and any block of no definition, hold no list)
  let (_, plain) = self::dump(
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able \\and {Bob} Baker}\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!(blocks(&plain)[0].get("declarations"), None);
}

/// Gap 8's span: a Declaration starts at what leads to its definer in the pending cover, so that a read of the span's content
/// is the definition: `\expandafter` (read by the gullet, whose `\csname` the span would run alone without it), `\long`
/// and `\global` (prefixes, whose meaning the span would lose). No earlier: not over a font switch or an earlier
/// declaration (each a call of its own), nor out into a conditional (the `\fi` is past the span: unbalanced). A box after a
/// declaration does not take it (`\def\u{U}\textbf{Cat}`). A macro defining one name twice lists it once. A font switch
/// is none, though `\itshape` reads as the document's (`\reinstall@nfss@defs`): `perfect_kernel_batchz2`.
#[test]
fn a_declarations_span_holds_what_leads_to_it() {
  if red_at_two("loses \"2 Q R ST U V\" from the placed frontmatter, a content-check Warning") {
    return;
  }
  let (_, dump) = dump(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/event_declaration_span_holds_its_lead.tex"
  ));
  assert_eq!(
    blocks(&dump)[0]["declarations"],
    json!([
      {"kind": "Declaration", "span": [0, 9], "label": "\\ww"},
      {"kind": "Declaration", "span": [18, 25], "label": "\\q", "role": "global"},
      {"kind": "Declaration", "span": [36, 42], "label": "\\r", "role": "global"},
      {"kind": "Declaration", "span": [53, 58], "label": "\\s"},
      {"kind": "Declaration", "span": [58, 64], "label": "\\t", "role": "global"},
      {"kind": "Declaration", "span": [77, 82], "label": "\\u"},
      {"kind": "Declaration", "span": [97, 102], "label": "\\v"},
      {"kind": "Declaration", "span": [113, 114], "label": "\\y"},
    ])
  );
}

/// Gap 9: a rehearsal goes on past an error it holds, so the block keeps all its events (the kinds that give none: a Fatal,
/// a run not undone, no journal, unchanged). The event the error was raised in says so, the name after it does not, and the
/// block lists it by position with its log line. The document raises the same error itself, once.
#[test]
fn a_held_error_marks_the_events_it_touches() {
  let (log, _, dump) = run(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/event_held_error_marks_its_events.tex"
  ));
  if red_at_two(
    "glues the undefined \\nosuch onto \"Able\" (\"nosuchAble\"), 2 false content-check Warnings",
  ) {
    return;
  }
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  let block = blocks(&dump)[0];
  assert_eq!(block["errors"], json!(1));
  assert_eq!(
    block["held"],
    json!([{"span": [4, 5], "message": "Error:undefined:\\nosuch The token T_CS[\\nosuch] is not defined."}])
  );
  assert_eq!(of_kind(&block["events"], "Name"), vec![
    json!({"kind": "Name", "span": [0, 9], "text": "Ann Able", "held": true}),
    json!({"kind": "Name", "span": [11, 20], "text": "Bob Baker"}),
  ]);
}

/// Gap 9's witnesses' shape (2201.04449's `\corref{corresponding_author}`, the rehearsal's request constructor digesting
/// the label's `_`): a held error the document never raises. The block's Mark says so; its twin block, a label of no `_`,
/// holds no error; the writer's reading is held back as before (`dump["writer"]` none: `written_resolution`).
#[test]
fn a_held_error_of_the_rehearsal_alone_marks_its_mark() {
  let (_, dump) = dump(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/event_held_error_binding_keeps_the_label.tex"
  ));
  let (held, clean) = (blocks(&dump)[0], blocks(&dump)[1]);
  assert_eq!(
    held["held"],
    json!([{"span": [13, 14], "message": "Error:unexpected:_ Script _ can only appear in math mode"}])
  );
  assert_eq!(of_kind(&held["events"], "Mark"), vec![json!({
    "kind": "Mark", "span": [8, 17], "label": "cor:cor_one", "role": "request", "symbol": false, "line_first": false,
    "held": true,
  })]);
  assert_eq!((&clean["errors"], clean.get("held")), (&json!(0), None));
  // (the twin block's own positions, per call: F1)
  assert_eq!(of_kind(&clean["events"], "Mark"), vec![json!({
    "kind": "Mark", "span": [9, 17], "label": "cor:cortwo", "role": "request", "symbol": false, "line_first": false,
  })]);
  if red_at_two("old engine only: the dump's writer outcome is null (=2 reads \"written\")") {
    return;
  }
  assert_eq!(dump["writer"], Value::Null);
}
