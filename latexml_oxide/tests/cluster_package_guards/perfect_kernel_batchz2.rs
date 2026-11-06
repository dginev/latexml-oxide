//! Guards for Z2 (`docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`, "Gaps in the event model", 3, 4, 5, 7, 10; revz01's findings on
//! gap 8): what the observer's events gain, read from the digested list and from the rehearsal's notes, and the readings the
//! writer's regressions needed. Each guard is a different code path: `a_font_switch_…` whose assignment is a declaration,
//! `a_definition_is_…` what holds it, `a_declarations_lead_…` where its span starts; `a_break_says…` a Break's kind, `a_break_a_binding…`
//! the notes' list beside the events; `a_break_and_a_comma_…`, `an_ordinal_…`, `a_name_says_…`, `an_orcid_…` one event field each.

use serde_json::{Value, json};

use super::perfect_kernel_batchz1::{blocks, dump, of_kind};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Finding 1 of revz01. A font switch assigns the kernel's font state and declares nothing, however it is reached: a size switch
/// then a shape switch (2608.17767's `\small\itshape`; the shape switches are made at `\begin{document}`, so they read as the
/// document's, `\reinstall@nfss@defs`), a bare `\selectfont`, a paper's macro of either (`\lxaff`, `\lxsel`). No list of names
/// says so, so a paper's own definition stands whatever its name (`\f@ox`, `\f@oo`, `\myf@size`, a name of slashes). A paper's macro
/// switching a size and defining declares its definition alone (`\lxmix`), and what a kernel macro was handed is the paper's
/// (`\newcount\nn`, `\newsavebox\bb`), not its own state.
#[test]
fn a_font_switch_declares_nothing_and_a_name_is_no_reason() {
  let (_, dump) = dump(repro!("event_declaration_font_switch_is_none"));
  assert_eq!(
    blocks(&dump)[0]["declarations"],
    json!([
      {"kind": "Declaration", "span": [94, 99], "label": "\\f@ox"},
      {"kind": "Declaration", "span": [110, 115], "label": "\\f@oo"},
      {"kind": "Declaration", "span": [126, 131], "label": "\\myf@size"},
      {"kind": "Declaration", "span": [142, 143], "label": "\\zz"},
      {"kind": "Declaration", "span": [153, 167], "label": "\\a/b/c/d"},
      {"kind": "Declaration", "span": [185, 187], "label": "\\nn", "role": "global"},
      {"kind": "Declaration", "span": [197, 199], "label": "\\bb", "role": "global"},
    ])
  );
}

/// Findings 2, 3 of revz01. A definition right before a box stays the block's, though that box's cover holds the boxless
/// invocation before it: a line break, a column, a skip, a rule, a box. One inside what a token of the field is still digesting is
/// that box's: an environment's (`minipage`, nested `center`, before or after its first ink, whose hooks are no definees either),
/// `\mbox`, `\parbox`, a formula, `\sbox`. A plain group's is the block's at its depth (`nest`).
#[test]
fn a_definition_is_the_blocks_unless_a_box_holds_it() {
  let (_, dump) = dump(repro!("event_declaration_beside_a_box"));
  assert_eq!(
    blocks(&dump)[0]["declarations"],
    json!([
      {"kind": "Declaration", "span": [3, 8], "label": "\\xa"},
      {"kind": "Declaration", "span": [19, 24], "label": "\\xb"},
      {"kind": "Declaration", "span": [33, 38], "label": "\\xc"},
      {"kind": "Declaration", "span": [44, 49], "label": "\\xd"},
      {"kind": "Declaration", "span": [68, 73], "label": "\\xe"},
      {"kind": "Declaration", "span": [81, 86], "label": "\\xf"},
      {"kind": "Declaration", "span": [95, 100], "label": "\\xg", "nest": 1},
    ])
  );
}

/// Finding 4 of revz01. What leads to a definition is what was invoked right before it, not what stands before it in the field: a
/// `\global` a `\let` read or a test read leads nothing (no overlap, no start inside the test); a `\global` before a `\relax`
/// still waits for its command, and through an `\expandafter` or a paper's macro too. A chain of `\expandafter`s is whole, and
/// the name after it no box's reach.
#[test]
fn a_declarations_lead_is_invocations_not_positions() {
  let (_, dump) = dump(repro!("event_declaration_lead_by_invocation"));
  assert_eq!(
    blocks(&dump)[0]["declarations"],
    json!([
      {"kind": "Declaration", "span": [0, 3], "label": "\\xa"},
      {"kind": "Declaration", "span": [3, 8], "label": "\\ya"},
      {"kind": "Declaration", "span": [17, 22], "label": "\\yb"},
      {"kind": "Declaration", "span": [29, 36], "label": "\\yc", "role": "global"},
      {"kind": "Declaration", "span": [42, 52], "label": "\\yd"},
      {"kind": "Declaration", "span": [59, 69], "label": "\\ye", "role": "global"},
      {"kind": "Declaration", "span": [75, 77], "label": "\\sa", "role": "global"},
      {"kind": "Declaration", "span": [83, 88], "label": "\\yf"},
      {"kind": "Declaration", "span": [88, 94], "label": "\\yg", "role": "global"},
    ])
  );
}

/// Gap 3 on the events. A Break says `par` when `\par` made it, which a `c` column does not print (tex.web §1094), whatever its role;
/// a line break says nothing. The block's names, as ever, part at either. The article's line reads every word; `\par` in its `\author`
/// comes through a macro, as the kernel's `\author` is not `\long`.
#[test]
fn a_break_says_what_made_it() {
  let (_, dump) = dump(repro!("event_break_says_what_made_it"));
  assert_eq!(of_kind(&blocks(&dump)[0]["events"], "Break"), vec![
    json!({"kind": "Break", "span": [8, 10]}),
    json!({"kind": "Break", "span": [14, 15], "break": "par"}),
    json!({"kind": "Break", "span": [18, 20]}),
    json!({"kind": "Break", "span": [23, 24]}),
    json!({"kind": "Break", "span": [27, 28]}),
    json!({"kind": "Break", "span": [36, 37], "role": "group", "break": "par"}),
  ]);
}

/// Gap 3 beside the events: the commands a binding declares a block break (`Policy::block_breaks`) are noted where they are called,
/// whatever the class makes them print, and do not part the events: the kernel's `\alignauthor`, made a tabular's end by the
/// document as sig-alternate.cls does, and ijcai's `\emails`, a no-op in its binding, whose name run goes on across it. A
/// control twin: `\affiliations` ends the names at the binding's own split, so it is called in no block; a block of no such call has
/// no list.
#[test]
fn a_break_a_binding_declares_stands_beside_the_events() {
  let (_, column) = dump(repro!("event_break_a_binding_declares_column"));
  let column_block = blocks(&column)[0];
  assert_eq!(
    column_block["breaks"],
    json!([
      {"kind": "Break", "span": [0, 1], "label": "\\alignauthor", "break": "column"},
      {"kind": "Break", "span": [18, 19], "label": "\\alignauthor", "break": "column"},
    ])
  );
  let (_, section) = dump(repro!("event_break_a_binding_declares_section"));
  let section_block = blocks(&section)[0];
  assert_eq!(
    section_block["breaks"],
    json!([{"kind": "Break", "span": [8, 9], "label": "\\emails", "break": "section"}])
  );
  assert_eq!(of_kind(&section_block["events"], "Name"), vec![
    json!({"kind": "Name", "span": [0, 18], "text": "Ann AbleBob Baker"})
  ]);
  let (_, plain) = self::dump(
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able \\and Bob Baker}\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!(blocks(&plain)[0].get("breaks"), None);
}

/// Gap 4. A comma or a Break says how many braces are printed and open before it, whatever printed them (`\{`, a formula's `\lbrace`,
/// `\textbraceleft`), a line break no end of them (`\{ann, bob,\\ cat\}@x.org`); after their close, and in a TeX group that prints
/// none, nothing.
#[test]
fn a_break_and_a_comma_say_their_printed_braces() {
  let (_, dump) = dump(repro!("event_printed_brace_depth"));
  let events = blocks(&dump)[0]["events"].as_array().expect("events");
  let seps: Vec<Value> = events
    .iter()
    .filter(|e| e["kind"] == "Break" || e["sep"] == "comma")
    .map(|e| json!([e["kind"], e["braces"]]))
    .collect();
  assert_eq!(seps, vec![
    json!(["Sep", 1]),
    json!(["Sep", 1]),
    json!(["Break", 1]),
    json!(["Sep", null]),
    json!(["Sep", null]),
    json!(["Sep", 1]),
    json!(["Sep", 1]),
  ]);
}

/// Gap 5. A number's suffix says so, glued or spaced (`5 \textsuperscript{th}`, 2609.01831's comment), in text or in a formula's
/// script (`21$^{st}$`, `$4^{th}$`); `10$^{a}$` is a mark and so is `Cat\textsuperscript{st}`, on no number.
#[test]
fn an_ordinal_says_so() {
  let (_, dump) = dump(repro!("event_ordinal_suffix"));
  let marks: Vec<Value> = of_kind(&blocks(&dump)[0]["events"], "Mark")
    .iter()
    .map(|mark| json!([mark["label"], mark["ordinal"]]))
    .collect();
  assert_eq!(marks, vec![
    json!(["st", true]),
    json!(["th", true]),
    json!(["st", true]),
    json!(["th", true]),
    json!(["a", null]),
    json!(["1", null]),
    json!(["st", null]),
  ]);
}

/// Gap 7 (100a's RED, hep-ex0106079). A Name says the labels its words print by hand, mid-line or leading it, with their span; `(c)`,
/// `2)` inside a bracket and `e)f` are none. Nothing reads them yet: the Cut's entries are as they were.
#[test]
fn a_name_says_its_printed_labels() {
  let (_, dump) = dump(repro!("event_printed_labels"));
  let names = of_kind(&blocks(&dump)[0]["events"], "Name");
  let labelled: Vec<&Value> = names
    .iter()
    .filter(|name| name.get("printed_labels").is_some())
    .collect();
  assert_eq!(labelled, vec![
    &json!({
      "kind": "Name", "span": [10, 48], "text": "a) also at UCEH b) on leave (c) of Lab",
      "printed_labels": [
        {"label": "a", "word": 0, "line_first": true, "span": [10, 12]},
        {"label": "b", "word": 4, "line_first": false, "span": [26, 28]},
      ],
    }),
    &json!({
      "kind": "Name", "span": [50, 54], "text": "d) X",
      "printed_labels": [{"label": "d", "word": 0, "line_first": true, "span": [50, 52]}],
    }),
  ]);
  assert_eq!(names[names.len() - 2]["text"], "Paper (see Fig. 2)");
}

/// Gap 10 (2408.15810, 2308.02269). llncs's `\orcidID` is a Contact of role `orcid` in the block's events, its span the call, no name's
/// text, and the reading holds it as the name's. An ORCID in an API creator's content reads as a Contact there too, though the resolver's
/// API name still prints it (the plan's Z13).
#[test]
fn an_orcid_is_a_contact_in_the_events() {
  let (_, dump) = dump(repro!("event_orcid_is_a_contact"));
  let contact = |span: [u16; 2], id: &str| json!({"kind": "Contact", "span": span, "text": id, "role": "orcid"});
  let events: Vec<Value> = blocks(&dump)
    .iter()
    .flat_map(|block| of_kind(&block["events"], "Contact"))
    .collect();
  assert_eq!(events, vec![
    contact([12, 33], "0000-0001-2345-6789"),
    contact([47, 68], "0000-0002-2345-6789"),
  ]);
  let names: Vec<(&str, &str)> = dump["resolved"]["creators"]
    .as_array()
    .expect("creators")
    .iter()
    .map(|c| {
      (
        c["name"].as_str().expect("a name"),
        c["contacts"][0]["text"].as_str().expect("a contact"),
      )
    })
    .collect();
  assert_eq!(names, vec![
    ("Ann Able", "0000-0001-2345-6789"),
    ("Bob Baker", "0000-0002-2345-6789"),
  ]);
}
