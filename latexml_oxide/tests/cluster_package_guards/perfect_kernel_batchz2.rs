//! Guards for Z2 (`docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`, "Gaps in the event model", 3, 4, 5, 7, 10; revz01's findings on
//! gap 8): what the observer's events gain, read from the digested list and from the rehearsal's notes, and the readings the
//! writer's regressions needed. Each guard is a different code path: `a_font_switch_…` whose assignment is a declaration,
//! `a_definition_is_…` what holds it, `a_declarations_lead_…` where its span starts; `a_break_says…` a Break's kind, `a_break_a_binding…`
//! the notes' list beside the events; `a_break_and_a_comma_…`, `an_ordinal_…`, `a_name_says_…`, `an_orcid_…` one event field each.
//! `a_kernel_macro_a_papers_macro_…` (Z2r2) what a paper's macro declares inside a kernel's; `a_declarations_end_…` where its span ends;
//! `the_last_author_call_wins_…` a binding's author body replaced, `an_address_two_entries_…` the same address under two entries.

use serde_json::{Value, json};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz1::{blocks, dump, of_kind},
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

/// S1 of revZ2. Within a kernel's macro a paper's macro reached, what the paper's macro's body or arguments name is the paper's (`\mcount`,
/// `\mboxes`, `\mform{\qd}`, the conditionals' branches, one level down `\mouter`), and the kernel's own assignments are not (`\mstar`'s
/// `\@let@token`, `\mfont`'s font state). `\@namedef`'s run has ended when its `\def` comes (`\mbuilt`, `\qk`), so that built name reads.
#[test]
fn a_kernel_macro_a_papers_macro_reaches_declares_what_it_names() {
  let (_, dump) = dump(repro!("event_declaration_through_a_kernel_macro"));
  let labels: Vec<&str> = blocks(&dump)[0]["declarations"]
    .as_array()
    .expect("declarations")
    .iter()
    .map(|declaration| declaration["label"].as_str().expect("a label"))
    .collect();
  assert_eq!(labels, [
    "\\qa", "\\qb", "\\qc", "\\qd", "\\qe", "\\qf", "\\qh", "\\qi", "\\qj", "\\qk"
  ]);
}

/// S2 of revZ2. A definer whose next token is no later one of the field (a macro's own `\relax`, the end of an argument) read as far as
/// the call around it: its span holds the whole definition, for a paper's macro, a `\textbf`, the last definition of an `\AtBeginDocument`.
#[test]
fn a_declarations_end_holds_what_it_read_inside_a_call() {
  let (_, dump) = dump(repro!("event_declaration_end_in_a_call"));
  assert_eq!(
    blocks(&dump)[0]["declarations"],
    json!([
      {"kind": "Declaration", "span": [2, 8], "label": "\\xa"},
      {"kind": "Declaration", "span": [10, 16], "label": "\\xb", "nest": 1},
      {"kind": "Declaration", "span": [18, 23], "label": "\\xc"},
      {"kind": "Declaration", "span": [23, 29], "label": "\\xd"},
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
/// whatever the class makes them print, and do not part the events: OmniBus's no-op `\alignauthor`, and ijcai's `\emails`, a no-op in
/// its binding, whose name run goes on across it. A control twin: `\affiliations` ends the names at the binding's own split, so it is
/// called in no block; a block of no such call has no list, and the kernel declares none, so a document's own `\alignauthor` has none.
#[test]
fn a_break_a_binding_declares_stands_beside_the_events() {
  let (_, column) = dump(repro!("event_break_a_binding_declares_column"));
  let column_block = blocks(&column)[0];
  assert_eq!(
    column_block["breaks"],
    json!([
      {"kind": "Break", "span": [0, 1], "label": "\\alignauthor", "break": "column"},
      {"kind": "Break", "span": [10, 11], "label": "\\alignauthor", "break": "column"},
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
  let (_, plain) = dump(
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able \\and Bob Baker}\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!(blocks(&plain)[0].get("breaks"), None);
  let (_, own) = dump(
    "\\documentclass{article}\n\\newcommand\\alignauthor{\\end{tabular}\\hfill\\begin{tabular}[t]{c}}\n\\begin{document}\n\\title{T}\n\\author{Ann Able \\alignauthor Bob Baker}\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!(blocks(&own)[0].get("breaks"), None);
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

/// The creators of `xml` (`perfect_kernel_batch84b::creators`) and the reading's: each its name with its printed contacts.
fn output_and_reading(xml: &str, dump: &Value) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
  let reading = dump["resolved"]["creators"]
    .as_array()
    .expect("creators")
    .iter()
    .map(|c| {
      std::iter::once(c["name"].as_str().expect("a name").to_string())
        .chain(
          ["affiliations", "contacts"]
            .iter()
            .flat_map(|list| c[*list].as_array().expect("a list"))
            .map(|a| a["text"].as_str().expect("its text").to_string()),
        )
        .collect()
    })
    .collect();
  (creators(xml), reading)
}

/// `rows` as what `creators` gives.
fn rows(rows: &[&[&str]]) -> Vec<Vec<String>> {
  rows
    .iter()
    .map(|row| row.iter().map(|s| (*s).to_string()).collect())
    .collect()
}

/// Part 3, 1702.08280 (`\tiny{ … M.~de~Naurois`, aa's `~` a control space, the writer's "M. de" and "Naurois"). A control space is its
/// font's word space, whatever that is in em (`\tiny` 0.347, typewriter 0.5), one name at any size; a tie alike. Its twin: a
/// wider skip in those fonts parts two names.
#[test]
fn a_word_space_is_its_fonts_at_any_size() {
  let (xml, dump) = dump(repro!("reading_word_space_of_a_small_font_parts_no_name"));
  let (output, reading) = output_and_reading(&xml, &dump);
  let names = rows(&[
    &["M. de Naurois"],
    &["X. de Ab"],
    &["T. Le Du"],
    &["K. da Silva"],
    &["J. van Vu"],
    &["Ann Lee"],
    &["Bob Ray"],
    &["Cat Roe"],
    &["Dan Fox"],
  ]);
  assert_eq!(output, names, "{xml}");
  assert_eq!(reading, names);
}

/// Part 3, 2401.15006 (`$^6$A$^*$STAR`, the writer's "A" and a nameless creator's "* STAR"). A symbol glued between two words on a line a
/// mark leads is the word's, whatever else shows that symbol and however many words stand before it: the entry is whole, in the reading
/// and in the output. Its twin: on a line no mark leads it stays a mark, and the name before it a name of its own, for the reading.
#[test]
fn a_symbol_glued_inside_a_word_is_the_words() {
  let (xml, dump_inside) = dump(repro!(
    "reading_symbol_mark_glued_inside_a_word_is_the_words"
  ));
  let (output, reading) = output_and_reading(&xml, &dump_inside);
  assert_eq!(
    reading,
    rows(&[
      &["Ann Able", "A∗STAR, Singapore"],
      &["Bob Baker", "R†D Lab"],
      &["Cat Cole", "IHPC A∗STAR"],
    ])
  );
  assert_eq!(
    output,
    rows(&[
      &["Ann Able", "affiliation=A * STAR, Singapore"],
      &["Bob Baker", "affiliation=R † D Lab"],
      &["Cat Cole", "affiliation=IHPC A ∗ STAR"],
    ]),
    "{xml}"
  );
  let (_, twin) = dump(repro!(
    "reading_symbol_mark_glued_to_a_name_with_a_legend_stays_a_mark"
  ));
  let labels: Vec<Value> = of_kind(&blocks(&twin)[0]["events"], "Mark")
    .iter()
    .map(|mark| mark["label"].clone())
    .collect();
  assert_eq!(labels, vec![json!("∗"), json!("∗")]);
  let names: Vec<&Value> = twin["resolved"]["creators"]
    .as_array()
    .expect("creators")
    .iter()
    .map(|c| &c["name"])
    .collect();
  assert_eq!(names, vec![&json!("Cher"), &json!("Bob Baker")]);
}

/// Part 3, 2606.16069's first call's shape. An address every carrier spells alone, under each by an entry of its own, is each entry's;
/// the ordinal on a names line stays the name's prefix. Its twin: one entry two people of one surname carry, the address spelling
/// both, stays the block's, in the output.
#[test]
fn an_address_two_entries_print_is_each_entrys() {
  let (xml, dump_prints) = dump(repro!("reading_same_address_under_each_of_two_entries"));
  let (output, reading) = output_and_reading(&xml, &dump_prints);
  assert_eq!(
    reading,
    rows(&[&["1 Ann Lee", "Univ A", "lee@a.org"], &[
      "2 Bob Lee",
      "Univ B",
      "lee@a.org"
    ],])
  );
  assert_eq!(
    output,
    rows(&[
      &["1 st Ann Lee", "affiliation=Univ A", "email=lee@a.org"],
      &["2 nd Bob Lee", "affiliation=Univ B", "email=lee@a.org"],
    ]),
    "{xml}"
  );
  let (xml, _) = dump(repro!(
    "reading_shared_address_two_names_spell_stays_the_blocks"
  ));
  assert_eq!(
    creators(&xml),
    rows(&[
      &["Ann Lee", "affiliation=Univ A"],
      &["Bob Lee", "affiliation=Univ A"],
      &["email=lee@a.org"]
    ]),
    "{xml}"
  );
}

/// 2606.16069 (pdflatex and Perl 4 creators, ours 8). The last `\author` wins in the blocks path too, whatever the earlier call queued:
/// its creators, their notes, their contacts; what stands beside the calls stays. Its twins: a plain `\author` after blocks, and a
/// call after a `\maketitle` that printed the one before it (both shown).
#[test]
fn the_last_author_call_wins_in_the_blocks_path() {
  let (xml, last) = dump(repro!("ieee_last_author_call_wins"));
  let (output, reading) = output_and_reading(&xml, &last);
  assert_eq!(reading, rows(&[&["Bob Baker", "Univ B", "bob@b.org"]]));
  assert_eq!(
    output,
    rows(&[&["Bob Baker", "affiliation=Univ B", "email=bob@b.org"]]),
    "{xml}"
  );
  assert!(xml.contains("Funded."), "{xml}");
  assert!(!xml.contains("First call"), "{xml}");
  let doc = |body: &str| {
    format!(
      "\\documentclass{{IEEEtran}}\n\\begin{{document}}\n\\title{{T}}\n{body}\n\\maketitle\nBody.\n\\end{{document}}\n"
    )
  };
  let (xml, _) = dump(&doc(
    r"\author{\IEEEauthorblockN{Bob Baker}\IEEEauthorblockA{Univ B}}\author{Ann Able}",
  ));
  assert_eq!(creators(&xml), rows(&[&["Ann Able"]]), "{xml}");
  let (xml, _) = dump(&doc(
    r"\author{\IEEEauthorblockN{Ann Able}\IEEEauthorblockA{Univ A}}\maketitle\author{\IEEEauthorblockN{Bob Baker}\IEEEauthorblockA{Univ B}}",
  ));
  assert_eq!(
    creators(&xml),
    rows(&[&["Ann Able", "affiliation=Univ A"], &[
      "Bob Baker",
      "affiliation=Univ B"
    ]]),
    "{xml}"
  );
}

/// Part 3, gap 5's reader (2609.01831's cited shape). An ordinal is its entry's text, on a block's labelled line, glued, by a formula, at
/// an entry's start, and in a call's content, whose reading today's view now reads alike, so no paper is declined for it; on a names line
/// it stays the name's prefix, as `an_address_two_entries_print_is_each_entrys` reads.
#[test]
fn an_ordinal_is_its_entrys_text() {
  let (xml, entry) = dump(repro!("reading_ordinal_in_an_entry_is_its_text"));
  let (output, reading) = output_and_reading(&xml, &entry);
  assert_eq!(
    reading,
    rows(&[
      &["Ann Able", "Univ A, 5 th Floor"],
      &["Bob Baker", "Univ B, 21 st Street"],
      &["Cat Cole", "3 rd Lab, Town"],
    ])
  );
  assert_eq!(
    output,
    rows(&[
      &["Ann Able", "affiliation=Univ A, 5 th Floor"],
      &["Bob Baker", "affiliation=Univ B, 21 st Street"],
      &["Cat Cole", "affiliation=3 rd Lab, Town"],
    ]),
    "{xml}"
  );
  let (xml, call) = dump(repro!("reading_ordinal_in_a_call_reads_as_todays"));
  let (output, reading) = output_and_reading(&xml, &call);
  assert_eq!(
    reading,
    rows(&[&["Ann Able", "Univ A, 5 th floor"], &[
      "Bob Baker",
      "Univ B, 24 th Street, 4 th Ave"
    ]])
  );
  assert_eq!(
    output,
    rows(&[&["Ann Able", "affiliation=Univ A, 5 th floor"], &[
      "Bob Baker",
      "affiliation=Univ B, 24 th Street, 4 th Ave"
    ]]),
    "{xml}"
  );
  assert_eq!(call["writer"], "same");
}
