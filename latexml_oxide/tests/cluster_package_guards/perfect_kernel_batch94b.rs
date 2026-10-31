//! Guards for batch 94b: a raw file's in-place contact setter is the Frontmatter API's (OXIDIZED_DESIGN_DIVERGENCES
//! #936; `frontmatter_stores.rs` `reroute_in_place_contact_setters`). A paper's own class or style, read raw, that
//! defines `\address`, `\affil`, `\affiliation`, `\institute` or `\email` as a setter printing its argument as a title-page
//! line — in place (ae100prg.cls, ar-1col.cls) or appended to the author block its own `\author` writes (11lomcon.sty,
//! jetpl.cls) — no longer keeps the line out of the frontmatter: the setter is the API's, as the kernel's locked
//! `\author` is the kernel's, and the author's `$^1$` mark finds its entry. Anything else stays the class's (the
//! fail-safe, decided by the class's code: a mark, an inline face, a footnote, a `\message`), and the class's own
//! definition prints the call where it stands when no author creator carries it, once `\maketitle` placed the authors
//! (cpc-hepnp.cls's `\address`, `perfect_kernel_batch91a::cpc_hepnp_is_not_routed_until_91c`) and once the document
//! body has sealed them. An affiliation inside
//! `\author{}` that opens with a mark is the entry the mark labels in the observer's reading (`\lx@fm@obs@affil`,
//! `events::leading_marks`). Each guard converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` (a
//! stand-in file cut to the witness's definitions, written by `{filecontents*}`) under the shadow switch in process and
//! with the binary, eager and `--streaming` (`perfect_kernel_batch92a::assert_reading`), asserts the XML's creators and
//! the reading's tuples whole and no Error or Warning, and the default mode (the fleet's) writes the same creators.
//! Every repro compiles clean with pdflatex (TL 2025).
//!
//! Round 2 (review rev94b): an in-block affiliation of several marked entries is one entry per mark (B1); a setter that
//! prints its line where it is called is the API's only inside a creator's own content or as a line between paragraphs,
//! never inside a sentence or an abstract, nor when the class's own definition calls it (B3) — one that stores its line
//! for the title page is the API's wherever it is called — and every call inside one `\author{…}` decides alike (B4).
//!
//! Round 3 (review rev94b r2): a line between paragraphs is the API's only at the document body's own level or in the
//! box the authors were set in, never inside another box, note, list, theorem or cell (B3); entries of an in-block
//! affiliation the writer could not cut are the one entry its leading marks label, never a nameless creator's, and a cut
//! entry keeps the fonts and the line-end punctuation of its line (B1, N4); an `[optional]{mandatory}` setter's optional
//! argument is a caption, a mark or a key by the API's convention (B2); a class's own definition calls the class's other
//! setters directly, nothing left for its look-ahead (S3).
//!
//! Round 4 (review rev94b r3): a cut entry is digested in the font in effect where its cut starts, read from the
//! rehearsal's digested ink (`events::entry_font`), and an argument other than a group spanning two entries is no cut
//! (B1); a line leading with a mark no name requests, and a list of single-word keys, is in doubt — the class's line —
//! and the writer never writes a creator without a name (B2, B2b; `writer::writes_a_nameless_creator`, unit-tested
//! there); a kept class body's call of a rerouted setter runs the setter's meaning when the document redefined it (S3);
//! the authors' box is recorded by its frame's serial, whatever group the kernel's call stood in.
//!
//! Round 5 (review rev94b r4): a font the reading cannot tell never undoes a cut, a line break's box tells the font
//! after it, and a container spanning entries wraps each again (B1); a mark no author requests yet is decided where
//! `\maketitle` places the authors (S1); a class body calls a rerouted setter through one token (S2).
//!
//! Round 6 (review rev94b r5): a line in doubt prints where it is called, its call kept only by a `\maketitle` flush
//! one of whose authors requests the mark — the printed ink lifted then, when it is a whole paragraph of its own, else
//! the call dropped (BL1) — and dropped by any other flush (BL2); a container whose start's font is unknown is cut
//! plain, nested containers wrap each entry in both (BL3).

use std::{path::Path, process::Command};

use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::set_fm_shadow_override;
use serde_json::Value;

use super::{
  perfect_kernel_batch84b::text,
  perfect_kernel_batch92a::assert_reading,
  perfect_kernel_batch93a::{creators_with_notes, default_mode_creators, owned},
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// The default mode's log and XML (no shadow switch: what the fleet writes).
#[track_caller]
fn default_mode(tex: &str) -> (String, String) {
  let (log, xml, ()) = convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    || set_fm_shadow_override(None),
    |_| (),
  );
  (log, xml)
}

/// `tex` converted by the binary in a fresh directory (the stand-in class beside it, written by its `{filecontents*}`),
/// eager or `--streaming`: its log and XML.
#[track_caller]
fn binary(tex: &str, streaming: bool) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
      if streaming {
        "--streaming=true"
      } else {
        "--streaming=false"
      },
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("t.xml");
  (log, xml)
}

/// Each `ltx:contact` of the XML, in order, as `role[name]=text` (`role=text` when it has no name of its own but the
/// API's default, a caption ending in a colon: "Address:\u{a0}", "Email: ").
fn named_contacts(xml: &str) -> Vec<String> {
  xml
    .match_indices("<contact ")
    .map(|(at, _)| {
      let open_end = at + xml[at..].find('>').expect("contact tag");
      let attr = |key: &str| {
        xml[at..open_end]
          .split(&format!("{key}=\""))
          .nth(1)
          .and_then(|r| r.split('"').next())
          .unwrap_or("")
          .to_string()
      };
      let close = open_end + xml[open_end..].find("</contact>").expect("closed contact");
      let name = attr("name");
      let named = if name.is_empty() || name.trim_end().ends_with(':') {
        String::new()
      } else {
        format!("[{name}]")
      };
      format!(
        "{}{named}={}",
        attr("role"),
        text(&xml[open_end + 1..close])
      )
    })
    .collect()
}

/// The text of the document's body: everything after its frontmatter, the first `<section` or `<para` on.
fn body_text(xml: &str) -> String {
  let at = [xml.find("<section"), xml.find("<para")]
    .into_iter()
    .flatten()
    .min()
    .unwrap_or(xml.len());
  text(&xml[at..])
}

/// ae100prg.cls's in-place setters (1303.3004; jpconf.cls's `\address` is the same): `\address` prints its argument in an
/// `{indented}` list where it is called, and `\email` is an `\address` line after a `\vspace*` — no `\maketitle`. Each
/// marked line is the entry its mark labels, on the names showing the mark (`api:leading-mark`), the e-mail Ann Able's
/// (it spells her); no copy of the lines stays in the body. After `\section{Intro}` the author list is sealed: the same
/// `\email` is the class's line there, in the body (`\lx@class@contact`), not a contact.
#[test]
fn a_raw_class_in_place_address_is_the_api() {
  let tex = repro!("raw_class_in_place_address_is_the_api.tex");
  // (the marks the written addresses answer no longer shown on the names, as today's takes those its own entries answer
  // — review rev94a r2 S3, `hide_answered_marks`; each address keeps the mark it prints)
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "address=1 Univ A, Town, Country",
      "email=ann@a.edu",
    ],
    &["Bob Baker", "address=1 Univ A, Town, Country"],
    &["Cy Cole", "address=2 Univ B, City, Country"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ A, Town, Country | api:leading-mark",
    "  c email ann@a.edu | api:annotate:last owner:spelled",
    "C Bob Baker [1] | line:names sep:comma",
    "  a Univ A, Town, Country | api:leading-mark",
    "C Cy Cole [2] | line:names sep:word",
    "  a Univ B, City, Country | api:leading-mark",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  let (log, xml) = default_mode(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let body = body_text(&xml);
  assert!(
    !body.contains("Univ A") && !body.contains("ann@a.edu"),
    "{body}"
  );
  assert!(
    body.contains("Write to the desk: E-mail: desk@journal.org"),
    "{body}"
  );
}

/// 11lomcon.sty (hep-ph0312288; ws-ijmpd.cls's pair, jetpl.cls's `\@authoraddress` in cond-mat0611585): `\author` and
/// `\address` append their lines to one buffer, `\@aabuffer`, its `\maketitle` prints. A package's setter (the
/// `\lx@package@loaded@raw` hook), rerouted because the class's own `\author` — the definition the kernel's lock refused —
/// writes the same buffer: each marked line on the names requesting its mark, none left in a block in the body.
#[test]
fn a_raw_style_buffered_address_is_the_api() {
  let tex = repro!("raw_style_buffered_address_is_the_api.tex");
  // (the answered marks no longer shown on the names — review rev94a r2 S3)
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "address=1 Univ A, Town",
      "address=2 Univ B, City",
    ],
    &["Bob Baker", "address=3 Lab C, Place"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1,2] | line:names",
    "  a Univ A, Town | api:leading-mark",
    "  a Univ B, City | api:leading-mark",
    "C Bob Baker [3] | line:names sep:comma",
    "  a Lab C, Place | api:leading-mark",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  let (_, xml) = default_mode(tex);
  assert!(!body_text(&xml).contains("Univ"), "{xml}");
}

/// ar-1col.cls's `\affil` (1602.05576, 2405.08158), a `\par`-ended line written inside `\author{}`: the API's
/// affiliation, not ink glued into the last name. In the observer's reading the affiliation opens with its mark, which
/// labels it (`events::leading_marks`, as an API entry's leading mark does): the names showing `1` request it
/// (`ruling1:request`) — it was the last name's, beside it (`owner:adjacent`, the text "1Univ A, …"), and the writer
/// declined; now the reading is today's XML.
#[test]
fn an_affil_inside_the_author_is_labelled_by_its_leading_mark() {
  let tex = repro!("raw_class_affil_inside_author_is_the_api.tex");
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=Univ A, Town, Country; email: ann@a.edu",
    ],
    &[
      "Bob Baker",
      "affiliation=Univ A, Town, Country; email: ann@a.edu",
      "affiliation=Univ B, City, Country",
    ],
    &[
      "Cy Cole",
      "affiliation=Univ A, Town, Country; email: ann@a.edu",
    ],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names mark:after-comma",
    "  a Univ A, Town, Country, email: ann@a.edu | ruling1:request",
    "C Bob Baker [1,2] | line:names sep:comma mark:after-comma",
    "  a Univ A, Town, Country, email: ann@a.edu | ruling1:request",
    "  a Univ B, City, Country | ruling1:request",
    "C Cy Cole [1] | line:names sep:word",
    "  a Univ A, Town, Country, email: ann@a.edu | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// The fail-safe: a raw class's setter of these names that prints no line of its argument stays the class's — a mark
/// (agujournal2019.cls's `\affil{1}`), an inline face of an address inside a note (`\texttt`), a footnote (ecai.cls's
/// `\institute` = `\thanks`), a `\message` that prints nothing (dmtcs-episciences.cls's `\address`): the XML is the
/// class's, as before the reroute, and nothing is rerouted.
#[test]
fn inline_contact_setters_stay_the_class() {
  let tex = repro!("raw_class_inline_contact_setters_stay_the_class.tex");
  let (log, xml) = default_mode(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    !log.contains("contact setters that print a title-page line"),
    "{log}"
  );
  assert_eq!(
    creators_with_notes(&xml),
    owned(&[&["Ann Able 1", "note=Visiting; ann@a.edu"], &[
      "Bob Baker 2",
      "note=Univ X"
    ],])
  );
  assert!(!xml.contains("Old Lab"), "{xml}");
  // (the binary, eager and `--streaming`: the class's definitions alike — review rev94b S1)
  for streaming in [false, true] {
    let (log, xml) = binary(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      creators_with_notes(&xml),
      owned(&[&["Ann Able 1", "note=Visiting; ann@a.edu"], &[
        "Bob Baker 2",
        "note=Univ X"
      ],]),
      "streaming={streaming}"
    );
    assert!(!xml.contains("Old Lab"), "streaming={streaming}: {xml}");
  }
}

/// Review rev94b B1: an affiliation inside `\author{}` holding several marked entries (`\affil{$^1$Univ A…\\ $^2$Univ
/// B…}`, ar-1col's shape, 1602.05576, 2405.08158) is one entry per mark in the observer's reading, each labelled by its
/// own leading marks — cut as the API's reading cuts a call (`events::mark_led_pieces`; jetpl's `\address{$^1$…\\
/// $^2$…}`, cond-mat0611585) — and the writer cuts today's one contact into them (`plan::cut_block_lines`, the entry's
/// span its own ink past its marks): no longer one contact under the first mark, on every name, with the `2` dropped.
#[test]
fn an_affil_inside_the_author_of_several_marked_entries_is_cut_by_its_marks() {
  let tex = repro!("raw_class_affil_inside_author_several_marks.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ A, Town, Country"],
    &[
      "Bob Baker",
      "affiliation=Univ A, Town, Country",
      "affiliation=Univ B, City, Country",
    ],
    &["Cy Cole", "affiliation=Univ A, Town, Country"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names mark:after-comma",
    "  a Univ A, Town, Country | ruling1:request",
    "C Bob Baker [1,2] | line:names sep:comma mark:after-comma",
    "  a Univ A, Town, Country | ruling1:request",
    "  a Univ B, City, Country | ruling1:request",
    "C Cy Cole [1] | line:names sep:word",
    "  a Univ A, Town, Country | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// Review rev94b B3: content is never moved out of its sentence. A rerouted call is the API's only inside a creator's own
/// content or as a line between paragraphs, outside any other frontmatter entry, before the authors are placed (`\address{Univ A,
/// Town}`, both authors'); one inside a paragraph ("Write to the curator, `\email{…}`, until 2030.") or an abstract
/// before `\maketitle` (revtex's order) is the class's line where it stands, and so is the `\address` the class's own
/// `\email` calls as it runs, after a `\vspace*` that leaves the paragraph (ae100prg.cls:185, 1303.3004; its body
/// rewritten to call the class's `\address`, `class_bodies_call_class_copies`). It was Bob Baker's e-mail, the sentence
/// left "Write to the curator, , until 2030."; the body is asserted whole (review rev94b r2 N5).
#[test]
fn a_contact_in_running_text_stays_the_class() {
  let tex = repro!("raw_class_contact_in_running_text_stays_the_class.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "address=Univ A, Town"], &[
    "Bob Baker",
    "address=Univ A, Town",
  ]];
  assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Univ A, Town | api:annotate:run",
    "C Bob Baker [] | line:names sep:word",
    "  a Univ A, Town | api:annotate:run",
  ]);
  assert_eq!(default_mode_creators(tex), owned(xml));
  for streaming in [false, true] {
    let (log, xml) = binary(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      body_text(&xml),
      "E-mail: list@archive.org is the list. Write to the curator, E-mail: data@archive.org , until 2030. 1 1 §1 1 \
       Intro Body.",
      "streaming={streaming}"
    );
  }
}

/// A rerouted setter that stores its line for the title page (stacs.cls's `\address[key]{…}` appended to `\addresses`,
/// 0902.1737) is the API's wherever it is called while the authors are set (`\lx@class@contact@stored`): its class's
/// run prints nothing where it stands, and the kernel's `\maketitle` never prints the store — a call standing in
/// horizontal mode (0902.1737's after the class's third `\author` argument our kernel leaves as a paragraph) lost its
/// address. Its optional argument, a key the class prints as the author's mark (`\@exname{@#1}`), is the key the
/// `\author[key]` gave its author (`optional_contact`, `author:N`): no contact name.
#[test]
fn a_stored_setter_is_the_api_wherever_it_is_called() {
  let tex = repro!("raw_class_stored_address_keyed_option.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "address=Univ A, Town"], &[
    "Bob Baker",
    "address=Univ B, City",
  ]];
  assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Univ A, Town | api:label",
    "C Bob Baker [] | line:names",
    "  a Univ B, City | api:label",
  ]);
  for streaming in [false, true] {
    let (log, xml) = binary(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      named_contacts(&xml),
      ["address=Univ A, Town", "address=Univ B, City"],
      "streaming={streaming}"
    );
  }
}

/// Review rev94b B4 (1805.12376, ci2018.cls): every `\affil` inside one `\author{…}` is the API's. The first one inside
/// the kernel's second creator attached its annotation to the creator, which then no longer read as digesting (its
/// content no lone `PlaceKeeper`), and the next ran the class's definition — "EVE EGAN , Univ C gy" in a name. A creator
/// digests while its first content is its `PlaceKeeper` (`creator_entry_in_progress`).
#[test]
fn every_affil_inside_one_author_is_the_api() {
  let tex = repro!("raw_class_affils_inside_one_author.tex");
  let xml: &[&[&str]] = &[
    &["ANN ABLE", "affiliation=Univ A"],
    &["DAN DOE", "affiliation=Univ B"],
    &["EVE EGAN", "affiliation=Univ C"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C ANN ABLE [] | line:names",
    "  a Univ A | owner:adjacent",
    "C DAN DOE [] | line:names sep:word",
    "  a Univ B | owner:adjacent",
    "C EVE EGAN [] | line:names",
    "  a Univ C | owner:adjacent",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// The no-creator branch of `\lx@class@contact` (osameet2.sty:49 `\let\author\MEETauthor`, saj.cls's `\authors`:
/// 1710.05603, 2110.09949, 2212.09079): names that never become creators keep the class's line beside them, not on a
/// nameless creator — no creator, the address printed where the class prints it.
#[test]
fn a_class_whose_names_are_no_creators_keeps_its_address() {
  let tex = repro!("raw_class_no_author_creator_keeps_its_address.tex");
  for streaming in [false, true] {
    let (log, xml) = binary(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "streaming={streaming}: {log}"
    );
    assert!(!xml.contains("<creator"), "streaming={streaming}: {xml}");
    assert!(
      text(&xml).contains("Ann Able and Bob Baker Univ A, Town Body."),
      "streaming={streaming}: {xml}"
    );
  }
}

/// Each `ltx:contact`'s content, in order, as its markup (its `xml:id`s aside): the fonts a cut entry keeps.
fn contact_markup(xml: &str) -> Vec<String> {
  xml
    .match_indices("<contact ")
    .map(|(at, _)| {
      let open_end = at + xml[at..].find('>').expect("contact tag");
      let close = open_end + xml[open_end..].find("</contact>").expect("closed contact");
      let mut inner = xml[open_end + 1..close].to_string();
      while let Some(id) = inner.find(" xml:id=\"") {
        let end = id + 9 + inner[id + 9..].find('"').expect("closed id");
        inner.replace_range(id..=end, "");
      }
      inner
    })
    .collect()
}

/// Review rev94b r2 B3: a setter that prints its line where it is called is the API's only between paragraphs at the
/// document body's own level (`frontmatter_stores.rs` `at_the_body_level`) — never inside a box, a note, a list item, a
/// theorem, a minipage, a parbox, a quote or an alignment's cell, each binding a mode of its own: there the class's
/// line stays where it prints (ae100prg.cls's `\address`/`\email`, 1303.3004; a class's `\affil` in an author tabular).
/// The footnote kept its e-mail (it was Bob Baker's, the note empty), the theorem its address ("holds." alone), the cell
/// "Univ C" (Ann Able's affiliation). Distinct from `a_contact_in_running_text_stays_the_class`: the mode a box binds,
/// not the paragraph or the frontmatter entry the call stands in.
#[test]
fn a_contact_in_a_box_stays_the_class() {
  let tex = repro!("raw_class_contact_in_a_box_stays_the_class.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "address=Univ A, Town"], &[
    "Bob Baker",
    "address=Univ A, Town",
  ]];
  assert_eq!(default_mode_creators(tex), owned(xml));
  for streaming in [false, true] {
    let (log, xml) = binary(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      named_contacts(&xml),
      ["address=Univ A, Town", "address=Univ A, Town"],
      "streaming={streaming}"
    );
    assert_eq!(
      body_text(&xml),
      "Text before 1 1 footnote 1 E-mail: fn@x.org the title. Theorem 1 1 Theorem 1 Theorem 1 Lab T, Place holds. • \
       1st item E-mail: it@x.org E-mail: mp@x.org Lab X, Place Cy Cole Univ C Lab P, Place Lab Q, Place 1 1 §1 1 Intro \
       Body.",
      "streaming={streaming}"
    );
  }
}

/// Review rev94b r2 B1, N4: an in-block affiliation's entries are cut with the group and the switches their line opens
/// before its mark (`{\itshape $^1$Univ A, Town;}`) or after it (`$^2${\itshape Univ B}`, `$^3$\textbf{Lab C}`) —
/// `plan::span_tokens`: the second entry was cut at its ink, inside its group, no balanced run, and today's one contact
/// stayed on a nameless creator; the first lost its italic — and the punctuation printed at its line's end (the `;`,
/// `events::mark_led_pieces_keeping`). ar-1col.cls's `\affil` inside `\author{}` (1602.05576, 2405.08158).
#[test]
fn a_cut_entry_keeps_its_fonts_and_its_punctuation() {
  let tex = repro!("raw_class_affil_cut_keeps_its_font.tex");
  // (the reading's text spells the `;` as the comma it separates by, `events_text`; the written contact prints it)
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ A, Town;"],
    &["Bob Baker", "affiliation=Univ B , City"],
    &["Cy Cole", "affiliation=Lab C , Place"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names mark:after-comma",
    "  a Univ A, Town, | ruling1:request",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ B, City | ruling1:request",
    "C Cy Cole [3] | line:names sep:word",
    "  a Lab C, Place | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  let (log, xml) = default_mode(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(contact_markup(&xml), [
    "<text font=\"italic\">Univ A, Town;</text>",
    "<text font=\"italic\">Univ B</text>, City",
    "<text font=\"bold\">Lab C</text>, <text font=\"typewriter\">Place</text>",
  ]);
}

/// A repro, its XML's creators and its reading's tuples.
type Shape = (
  &'static str,
  &'static [&'static [&'static str]],
  &'static [&'static str],
);

/// Review rev94b r2 B1: entries of an in-block affiliation the writer could not cut — an entry of no text (`$^1$\\
/// $^2$Univ B`), entries whose texts are not the contact's (the word "and" between them), an entry no name requests
/// (`$^3$Lab C`) — are the one entry the leading marks label, today's one contact on the names requesting them, never a
/// nameless creator (`events::labelled_entries`, `lex_events`). ar-1col.cls's `\affil` (1602.05576, 2405.08158). This
/// pins the fallback, not a correct reading: its known misattribution stays — `noline`'s "Univ B, City" is on Ann
/// Able, whose `1` labels the content's first mark, though pdflatex prints it under Bob Baker's `2` (review rev94b r3
/// NIT); a nameless creator holding it was worse.
#[test]
fn entries_the_writer_cannot_cut_stay_one_contact() {
  let shapes: [Shape; 3] = [
    (
      repro!("raw_class_affil_uncut_noline_stays_one_contact.tex"),
      &[&["Ann Able", "affiliation=Univ B, City"], &["Bob Baker"]],
      &[
        "C Ann Able [1] | line:names mark:after-comma",
        "  a Univ B, City | ruling1:request",
        "C Bob Baker [2] | line:names sep:comma",
      ],
    ),
    (
      repro!("raw_class_affil_uncut_midword_stays_one_contact.tex"),
      &[
        &["Ann Able", "affiliation=Univ A, Town and Univ B, City"],
        &["Bob Baker"],
      ],
      &[
        "C Ann Able [1] | line:names mark:after-comma",
        "  a Univ A, Town and Univ B, City | ruling1:request",
        "C Bob Baker [2] | line:names sep:comma",
      ],
    ),
    (
      repro!("raw_class_affil_uncut_unrequested_stays_one_contact.tex"),
      &[
        &["Ann Able", "affiliation=Univ A, Town Univ B, City Lab C"],
        &["Bob Baker"],
      ],
      &[
        "C Ann Able [1] | line:names mark:after-comma",
        "  a Univ A, Town Univ B, City Lab C | ruling1:request",
        "C Bob Baker [2] | line:names sep:comma",
      ],
    ),
  ];
  for (tex, xml, shadow) in shapes {
    let reading = assert_reading(tex, Some(xml), shadow);
    assert_eq!(reading.3, Value::from("written"));
    assert_eq!(default_mode_creators(tex), owned(xml));
  }
}

/// Review rev94b r1-r2 B2: an `[optional]{mandatory}` setter's optional argument by the API's convention
/// (`frontmatter_stores.rs` `optional_contact`; Perl ams_support.sty.ltxml:115 `\lx@add@address[name={#1}]{#2}`): words
/// no creator requests are the line's caption — amsart copies' `\address[Current address]{…}` (smfart, nyjm,
/// birkmult, birkart, ECACRM), "Current address" was dropped —, an empty one is dropped, and a key the names' marks print
/// labels the line by that mark (`\address[a]{…}` under `Dan Doe$^{a}$`: both lines were on both later names).
#[test]
fn an_optional_caption_names_the_line() {
  let tex = repro!("raw_class_optional_address_is_named.tex");
  let contacts = [
    "address=Univ A, Town",
    "address[Current address]=Lab C, Place",
    "email=ann@a.edu",
    "address=Univ B, City",
    "address=Lab D, Place",
    "address=Lab E, Place",
  ];
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "address=Univ A, Town",
      "address=Lab C, Place",
      "email=ann@a.edu",
    ],
    &["Bob Baker", "address=Univ B, City"],
    &["Cy Cole", "address=Lab D, Place"],
    &["Dan Doe", "address=Lab E, Place"],
  ];
  assert_eq!(default_mode_creators(tex), owned(xml));
  for streaming in [false, true] {
    let (log, xml) = binary(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "streaming={streaming}: {log}"
    );
    assert_eq!(named_contacts(&xml), contacts, "streaming={streaming}");
  }
}

/// Review rev94b r1-r2 B2: an optional argument that is a printed mark (w-art.cls's `\address[\inst{1}]{…}`: 0910.5837,
/// 1105.1753, quant-ph0606237) leads the line, read by its mark as any marked line is — each address on the names
/// requesting it (today's XML as the reading: the mark labels the line, `\lx@affiliation@withinst`), the marks it
/// answers no longer shown on the names; both were on both names, the marks unanswered.
#[test]
fn an_optional_mark_labels_the_line() {
  let tex = repro!("raw_class_optional_mark_labels_the_address.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "address=Univ A, Town", "address=Lab B, City"],
    &["Bob Baker", "address=Lab B, City"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1,2] | line:names",
    "  a Univ A, Town | api:leading-mark",
    "  a Lab B, City | api:leading-mark",
    "C Bob Baker [2] | line:names",
    "  a Lab B, City | api:leading-mark",
  ]);
  // (today's XML the reading: the line's mark labels it, `\lx@affiliation@withinst`)
  assert_eq!(reading.3, Value::from("same"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// Review rev94b r1-r2 B2: a key an `\author[key]` gave its authors labels the line for each of them (`author:N`,
/// `\lx@author@keys`; Perl jheppub.sty.ltxml:35-37 `\author[]{}` → `annotations={#1}`, `\affiliation[]{}` →
/// `label={#1}`): a jheppub copy's `\affiliation[b]{McGill …}` (2303.16316) is Joydeep Chakravarty's (`\author[a,b]`)
/// alone — it was every author's, the second `annotate=run` affiliation the same run's as the first.
#[test]
fn an_optional_key_labels_its_authors() {
  let tex = repro!("raw_class_optional_key_labels_its_authors.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "affiliation=Univ A, Town"], &[
    "Bob Baker",
    "affiliation=Univ A, Town",
    "affiliation=Lab B, City",
  ]];
  assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Univ A, Town | api:label",
    "C Bob Baker [] | line:names",
    "  a Univ A, Town | api:label",
    "  a Lab B, City | api:label",
  ]);
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// `tex`'s creators in the default mode (the fleet's), and by the binary eager and `--streaming`: no Error or Warning,
/// each contact's markup `markup` and the body's text `body` when given.
#[track_caller]
fn assert_written(tex: &str, creators: &[&[&str]], markup: Option<&[&str]>, body: Option<&str>) {
  assert_eq!(default_mode_creators(tex), owned(creators));
  for streaming in [false, true] {
    let (log, xml) = binary(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "streaming={streaming}: {log}"
    );
    if let Some(markup) = markup {
      assert_eq!(contact_markup(&xml), markup, "streaming={streaming}");
    }
    if let Some(body) = body {
      assert_eq!(body_text(&xml), body, "streaming={streaming}");
    }
  }
}

/// Review rev94b r3 B1: an in-block affiliation's entry is digested in the font in effect where its cut starts — a
/// switch at the content's head (`\itshape`, `\footnotesize`), a font command's argument (`\textit{…}`), a font
/// environment, nested groups with a size and a colour (`{\small{\color{red}…}}`) — read from the rehearsal's digested
/// ink (`events::entry_font`) and carried by the entry (`plan::entry_font_tokens`, `\lx@fm@entry@font`): every entry
/// after a line break was plain. Each affiliation its own group in the rehearsal, as the API digests it (a switch of the
/// first is no font of the next). ar-1col.cls's `\affil` inside `\author{}` (1602.05576, 2405.08158). Distinct from
/// `a_cut_entry_keeps_its_fonts_and_its_punctuation`: the font is open from before the entry's line, not its own.
#[test]
fn a_cut_entry_keeps_the_font_open_before_it() {
  assert_written(
    repro!("raw_class_affil_cut_keeps_the_font_open_before_it.tex"),
    &[
      &["Ann Able", "affiliation=Univ A, Town"],
      &["Bob Baker", "affiliation=Univ B, City"],
      &["Cy Cole", "affiliation=Lab C, Place"],
      &["Dan Doe", "affiliation=Lab D, Place"],
      &["Eve Ern", "affiliation=Inst E, Town"],
      &["Fay Fox", "affiliation=Inst F, City"],
      &["Gil Gray", "affiliation=Dept G, Town"],
      &["Hal Hart", "affiliation=Dept H, City"],
      &["Ida Ince", "affiliation=Org I, Town"],
      &["Jo Jung", "affiliation=Org J, City"],
    ],
    Some(&[
      "<text font=\"italic\">Univ A, Town</text>",
      "<text font=\"italic\">Univ B, City</text>",
      "<text font=\"italic\">Lab C, Place</text>",
      "<text font=\"italic\">Lab D, Place</text>",
      "<text fontsize=\"80%\">Inst E, Town</text>",
      "<text fontsize=\"80%\">Inst F, City</text>",
      "<text font=\"italic\">Dept G, Town</text>",
      "<text font=\"italic\">Dept H, City</text>",
      "<text color=\"#FF0000\" fontsize=\"90%\">Org I, Town</text>",
      "<text color=\"#FF0000\" fontsize=\"90%\">Org J, City</text>",
    ]),
    None,
  );
}

/// Review rev94b r3 B1, the API's path (`plan::cut`, an API call's content cut by its marks): each piece in the font in
/// effect where it starts — the content's head switch (`\itshape $^1$…\\ $^2$…`) and an enclosing group's
/// (`{\bfseries $^3$…\\ $^4$…}`): the first had been on no piece, the second on the first piece only. Distinct from
/// `a_cut_entry_keeps_the_font_open_before_it`: an API call's rehearsal, not the author block's.
#[test]
fn an_api_piece_keeps_the_font_open_before_it() {
  assert_written(
    repro!("raw_class_address_cut_keeps_the_font_open_before_it.tex"),
    &[
      &["Ann Able", "address=1 Univ A, Town"],
      &["Bob Baker", "address=2 Univ B, City"],
      &["Cy Cole", "address=3 Lab C, Place"],
      &["Dan Doe", "address=4 Lab D, Place"],
    ],
    Some(&[
      "<sup>1</sup><text font=\"italic\">Univ A, Town</text>",
      "<sup>2</sup><text font=\"italic\">Univ B, City</text>",
      "<sup>3</sup><text font=\"bold\">Lab C, Place</text>",
      "<sup>4</sup><text font=\"bold\">Lab D, Place</text>",
    ]),
    None,
  );
}

/// Review rev94b r3 B1, r4 B1: a container other than a group holding two entries' ink — an `\underline{…}`, an
/// `\emph{…}` (pdflatex prints both lines in it) — wraps each cut entry again (`writer::container_wrap`): each name its
/// own line with the markup. A container the writer cannot wrap exactly (a switch inside it before its first entry,
/// `\emph{\bfseries $^5$…}`) is cut as before fonts were read: attribution first. Re-baselined in r5: r4 left each
/// affiliation one contact on the first name (the second name had nothing), worse than 94a r7's cut.
#[test]
fn entries_inside_one_container_are_each_wrapped_again() {
  assert_written(
    repro!("raw_class_affil_container_wraps_each_entry.tex"),
    &[
      &["Ann Able", "affiliation=Univ A, Town"],
      &["Bob Baker", "affiliation=Univ B, City"],
      &["Cy Cole", "affiliation=Lab C, Place"],
      &["Dan Doe", "affiliation=Lab D, Place"],
      &["Eve Ern", "affiliation=Lab E, Place"],
      &["Fay Fox", "affiliation=Lab F, Place"],
    ],
    Some(&[
      "<text class=\"ltx_underline\">Univ A, Town</text>",
      "<text class=\"ltx_underline\">Univ B, City</text>",
      "<emph font=\"italic\">Lab C, Place</emph>",
      "<emph font=\"italic\">Lab D, Place</emph>",
      "<text font=\"bold\">Lab E, Place</text>",
      "Lab F, Place",
    ]),
    None,
  );
}

/// Review rev94b r4 B1: a font the reading cannot tell where an entry's cut starts never undoes the cut — the entry is
/// cut as before fonts were read (the switches of the group it sits in), and a line break's own box tells the font in
/// effect where the next entry starts (`events::entry_font`): `$^1$\textbf{Univ A}\\ $^2$\textbf{Univ B}`, `\textit{Town}
/// \\`, `Town~\\` and `\itshape $^7$\textbf{Dept G}` each left the whole affiliation on the first name. ar-1col.cls's
/// `\affil` inside `\author{}` (1602.05576, 2405.08158). Distinct from `a_cut_entry_keeps_the_font_open_before_it`: the
/// fonts the reading cannot tell, not the ones it reads.
#[test]
fn an_uncertain_font_never_undoes_a_cut() {
  assert_written(
    repro!("raw_class_affil_uncertain_font_still_cuts.tex"),
    &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ B"],
      &["Cy Cole", "affiliation=Lab C, Town"],
      &["Dan Doe", "affiliation=Lab D , City"],
      &["Eve Ern", "affiliation=Inst E, Town"],
      &["Fay Fox", "affiliation=Inst F , City"],
      &["Gil Gray", "affiliation=Dept G , Town"],
      &["Hal Hart", "affiliation=Dept H, City"],
    ],
    Some(&[
      "<text font=\"bold\">Univ A</text>",
      "<text font=\"bold\">Univ B</text>",
      "Lab C, <text font=\"italic\">Town</text>",
      "<text font=\"bold\">Lab D</text>, City",
      "Inst E, Town",
      "<text font=\"italic\">Inst F</text>, City",
      "<text font=\"bold italic\">Dept G<text font=\"medium\">, Town</text></text>",
      "<text font=\"italic\">Dept H, City</text>",
    ]),
    None,
  );
}

/// Review rev94b r4 S1, r5 BL1: a title-block line leading with a mark no author requests yet prints where it is called,
/// its API call queued in doubt, decided at the `\maketitle` that places the authors (`frontmatter_stores.rs`
/// `doubtful`, `place.rs` `settle_calls_in_doubt`): a later `\author` requesting it takes it and the line's printed ink is
/// lifted (`settle_contacts_in_doubt`; Bob Baker's `\address[\inst{2}]`, Cy Cole's `\affil{\inst{3}…}`, both left in
/// the body by r4 — the body holds no copy), and one requested by none is the class's line where it printed, no creator
/// of no name's (94a r7's). w-art.cls's `\address[\inst{1}]` (0910.5837, 1105.1753, quant-ph0606237).
/// Distinct from `an_optional_mark_no_name_requests_stays_the_class`: an author after the line.
#[test]
fn a_mark_a_later_author_requests_is_the_api() {
  assert_written(
    repro!("raw_class_optional_mark_a_later_author_requests.tex"),
    &[&["Ann Able 1"], &["Bob Baker", "address=Univ B, City"], &[
      "Cy Cole",
      "affiliation=Lab C, Place",
    ]],
    Some(&["Univ B, City", "Lab C, Place"]),
    Some("( 4 ) Lab D, Place Body."),
  );
}

/// Review rev94b r3 B2: a contact line of its own leading with a mark no name requests — `\inst{3}`,
/// `\textsuperscript{4}`, `$^{5}$` under `Ann Able\inst{1} and Bob Baker\inst{2}` — is in doubt
/// (`frontmatter_stores.rs` `unrequested_mark`): the class's line where it is called, its words kept (the
/// main session's ruling: content over markup). "Lab C, Place" was a creator of no name's, the other two both authors'.
#[test]
fn a_line_no_name_requests_stays_the_class() {
  assert_written(
    repro!("raw_class_affil_unrequested_mark_stays_the_class.tex"),
    &[&["Ann Able", "affiliation=Univ A, Town"], &[
      "Bob Baker",
      "affiliation=Univ B, City",
    ]],
    None,
    Some("3 Lab C, Place 4 Lab D, Place 5 Lab E, Place Body."),
  );
}

/// Review rev94b r3 B2, B2b: an `[optional]{mandatory}` setter's optional mark no name requests (`\address[\inst{3}]`,
/// `\address[$^{4}$]`) and a list of single-word keys no name requests (`\address[c, d]`, captioned "c, d") are in
/// doubt (`optional_contact`): the class's line where it is called. w-art.cls's `\address[\inst{1}]` (0910.5837,
/// 1105.1753, quant-ph0606237) stays its authors'.
#[test]
fn an_optional_mark_no_name_requests_stays_the_class() {
  assert_written(
    repro!("raw_class_optional_unrequested_mark_stays_the_class.tex"),
    &[&["Ann Able", "address=Univ A, Town"], &[
      "Bob Baker",
      "address=Univ B, City",
    ]],
    None,
    Some("( 3 ) Lab C, Place ( 4 ) Lab D, Place (c, d) Lab E, Place Body."),
  );
}

/// Review rev94b r3 S3: a kept class body's call of a rerouted setter (ae100prg.cls's `\email` through its `\address`,
/// 1303.3004) runs the class's definition only while the setter means the reroute (`\lx@class@contact@dispatch@<name>`):
/// redefined in a group, then globally, it runs the document's — "Addr:", "Addr2:" were dropped.
#[test]
fn a_redefined_setter_runs_its_meaning() {
  assert_written(
    repro!("raw_class_redefined_setter_runs_its_meaning.tex"),
    &[&["Ann Able"], &["Bob Baker"]],
    None,
    Some(
      "Write to us, Addr: E-mail: d@x.org until 2030. Or write, E-mail: e@x.org now. Write again, Addr2: E-mail: \
       f@x.org later. Body.",
    ),
  );
}

/// Review rev94b r4 S2: a kept class body calls a rerouted setter through one token
/// (`\lx@class@contact@dispatch@<name>`), whatever reads it — cexp-like `\expandafter\address\expandafter{\emailprefix
/// #1}` (the braced dispatcher left the copy's `#1` to the `\expandafter`: "Addr:" alone, the e-mail in the sentence),
/// `\let\x\address`, `\@ifnextchar[\address` — and an `\ifx` operand is the class's own definition. ae100prg.cls's
/// `\email` (1303.3004). Distinct from `a_redefined_setter_runs_its_meaning`: the call site's shape, not the meaning.
#[test]
fn a_class_body_calls_a_setter_through_one_token() {
  assert_written(
    repro!("raw_class_setter_body_calls_through_one_token.tex"),
    &[&["Ann Able"], &["Bob Baker"]],
    None,
    Some(
      "Write to us, Addr: E-mail: d@x.org until 2030, Addr: Aff: Univ A then Addr: Inst: Lab C and Addr: [ Lab D] \
       end. Body.",
    ),
  );
}

/// Review rev94b r2-r3 S3: after `\maketitle` each call is the class's run, and the class's look-ahead — `\@ifstar`,
/// `\futurelet`, `\@ifnextchar[`, `\ignorespaces` — reads the document's own next token, nothing of the reroute's after
/// the call (`class_bodies_call_class_copies`).
#[test]
fn a_class_run_looks_ahead_at_the_documents_tokens() {
  assert_written(
    repro!("raw_class_lookahead_after_a_class_run.tex"),
    &[&["Ann Able"]],
    None,
    Some(
      "Contact a@x.org starred now, Lab A (dot). then Univ B ¡note¿ end Inst C x. 1 1 §1 1 Intro Body.",
    ),
  );
}

/// Review rev94b r3 placements: a call inside a `\marginpar`, a float or a `\caption` is the class's line where it
/// stands (`at_the_body_level`: each binds a mode of its own). Distinct from `a_contact_in_a_box_stays_the_class`:
/// the margin note, the float and the caption.
#[test]
fn a_contact_in_a_margin_note_float_or_caption_stays_the_class() {
  assert_written(
    repro!("raw_class_contact_in_a_margin_note_float_or_caption_stays_the_class.tex"),
    &[&["Ann Able", "address=Univ A, Town"], &[
      "Bob Baker",
      "address=Univ A, Town",
    ]],
    None,
    Some(
      "Text E-mail: m@x.org here. E-mail: n@x.org Lab F, Place Figure 1 1 Figure 1 X 1 Lab G, Place Figure 1 Lab G, \
       Place Body.",
    ),
  );
}

/// Review rev94b r3: the box the authors were set in is recorded globally by its frame's serial (`author_box`): an
/// imsart-like `\affiliation` after `\end{aug}`, still in the frontmatter's `\vbox` (1301.1466), is the API's — the
/// record had been the kernel call's group's, gone at its end, and the line stayed the class's.
#[test]
fn an_affiliation_after_the_author_group_in_its_box_is_the_api() {
  assert_written(
    repro!("raw_style_affiliation_after_the_author_group_in_its_box.tex"),
    &[&["Ann Able", "affiliation=Univ A, Town"], &[
      "Bob Baker",
      "affiliation=Univ A, Town",
    ]],
    None,
    Some("Body."),
  );
}

/// Review rev94b r5 BL1: a title-block line in doubt — a mark no author requests (`\address[\inst{9}]`), a key list
/// (`\address[c, d]`) — prints where it is called, as the class prints it (`frontmatter_stores.rs` `doubtful`), its
/// call queued in doubt beside it: r5 set every such line after the title, after the text printed between them, and
/// digested the footnote of one after the body's. The body's paragraphs whole, in pdflatex's order. A line holding a
/// footnote is not queued in doubt (`counts_or_labels`). w-art.cls's `\address[\inst{1}]` (0910.5837, 1105.1753,
/// quant-ph0606237). Distinct from `an_optional_mark_no_name_requests_stays_the_class`: text between the lines.
#[test]
fn a_line_in_doubt_prints_where_it_is_called() {
  assert_written(
    repro!("raw_class_line_in_doubt_prints_where_called.tex"),
    &[&["Ann Able", "address=Univ A, Town"]],
    None,
    Some(
      "( 9 ) Univ Z, Zed Pre text. (c, d) Lab C, Place We thank X. ( 8 ) Univ Y 1 1 footnote 1 Y note. Body. 2 2 \
       footnote 2 Body note.",
    ),
  );
}

/// Review rev94b r5 BL2: a flush other than a `\maketitle`'s — the fallback at a `\section` (a `\maketitle` after it
/// placing nothing more) or at `\end{document}` — drops a call in doubt (`place.rs` `settle_calls_in_doubt`): the line
/// stays where the class printed it, "(9)" with it; r5's fallback digested the call, a creator of no name holding
/// "Univ Z, Zed". w-art.cls's `\address[\inst{1}]` (0910.5837, 1105.1753, quant-ph0606237).
#[test]
fn a_fallback_flush_drops_a_call_in_doubt() {
  assert_written(
    repro!("raw_class_line_in_doubt_at_a_fallback_flush.tex"),
    &[&["Ann Able", "address=Univ A, Town"]],
    None,
    Some("( 9 ) Univ Z, Zed 1 1 §1 1 Intro Pre text. Body."),
  );
  assert_written(
    repro!("raw_class_line_in_doubt_at_the_end_flush.tex"),
    &[&["Ann Able", "address=Univ A, Town"]],
    None,
    Some("( 9 ) Univ Z, Zed Body."),
  );
}

/// Review rev94b r5 BL1, the fail-safe of the lift: a line in doubt a later author requests whose printed ink is no
/// paragraph of its own (`\par{\small …} ` runs into "More text.") is not lifted, its kept contact dropped where the
/// frontmatter is placed (`place.rs` `settle_contacts_in_doubt`) — the line where it printed, never twice nor moved;
/// and a line holding a footnote is not queued in doubt at all (kept, its call would digest the footnote again).
/// Distinct from `a_mark_a_later_author_requests_is_the_api`, where the ink is lifted.
#[test]
fn a_kept_line_whose_ink_cannot_be_lifted_stays_the_class() {
  assert_written(
    repro!("raw_class_line_in_doubt_kept_but_not_lifted.tex"),
    &[&["Ann Able 1"], &["Bob Baker 2"], &["Cy Cole 3"]],
    None,
    Some("( 2 ) Univ B, City More text. 3 Lab C 1 1 footnote 1 C note. Body."),
  );
}

/// Review rev94b r5 BL3: a container spanning entries whose start's font the boxes cannot tell (`\itshape\emph{…}`,
/// `\textcolor{red}{\emph{…}}`, `\small\emph{…}`) gives its entries neither font nor wrap — the base's cut
/// (`plan::entry_forms`): r5 wrapped each in `\emph` alone, italic where pdflatex prints upright, red and the size lost.
/// Nested containers wrap each entry in both (`plan::container_wrap`: the inner container's opening is no switch of the
/// outer's): `\emph{\underline{…\\…}}` was cut plain. ar-1col.cls's `\affil` inside `\author{}` (1602.05576,
/// 2405.08158). Distinct from `entries_inside_one_container_are_each_wrapped_again`: the font outside the container.
#[test]
fn a_container_whose_font_is_unknown_is_cut_plain() {
  assert_written(
    repro!("raw_class_affil_container_outer_font_unknown.tex"),
    &[
      &["Ann Able", "affiliation=Univ A, Town"],
      &["Bob Baker", "affiliation=Univ B, City"],
      &["Cy Cole", "affiliation=Lab C, Place"],
      &["Dan Doe", "affiliation=Lab D, Place"],
      &["Eve Ern", "affiliation=Inst E, Town"],
      &["Fay Fox", "affiliation=Inst F, City"],
      &["Gil Gray", "affiliation=Dept G, Town"],
      &["Hal Hart", "affiliation=Dept H, City"],
    ],
    Some(&[
      "Univ A, Town",
      "Univ B, City",
      "Lab C, Place",
      "Lab D, Place",
      "Inst E, Town",
      "Inst F, City",
      "<emph class=\"ltx_underline\" font=\"italic\">Dept G, Town</emph>",
      "<emph class=\"ltx_underline\" font=\"italic\">Dept H, City</emph>",
    ]),
    Some("Body."),
  );
}
