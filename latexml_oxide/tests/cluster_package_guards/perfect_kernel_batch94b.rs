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
//! for the title page is the API's wherever it is called — and every call inside one `\author{…}` decides alike (B4). An
//! optional argument is dropped, as OmniBus drops it (B2 deferred: `raw_class_optional_address_is_named`, a catalog
//! repro).

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
/// `\email` calls as it runs, after a `\vspace*` that leaves the paragraph (ae100prg.cls:185, 1303.3004;
/// `\lx@class@contact@begin`). It was Bob Baker's e-mail, the sentence left "Write to the curator, , until 2030.".
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
    let all = text(&xml);
    assert!(
      all.contains("Write to the curator, E-mail: data@archive.org , until 2030."),
      "streaming={streaming}: {all}"
    );
    assert!(
      all.contains("E-mail: list@archive.org is the list."),
      "streaming={streaming}: {all}"
    );
  }
}

/// A rerouted setter that stores its line for the title page (stacs.cls's `\address[key]{…}` appended to `\addresses`,
/// 0902.1737) is the API's wherever it is called while the authors are set (`\lx@class@contact@stored`): its class's
/// run prints nothing where it stands, and the kernel's `\maketitle` never prints the store — a call standing in
/// horizontal mode (0902.1737's after the class's third `\author` argument our kernel leaves as a paragraph) lost its
/// address. Its optional argument, a key the class prints as the author's mark (`\@exname{@#1}`), is dropped as OmniBus
/// drops it (OmniBus.cls.ltxml:92): no contact name.
#[test]
fn a_stored_setter_is_the_api_wherever_it_is_called() {
  let tex = repro!("raw_class_stored_address_keyed_option.tex");
  let xml: &[&[&str]] = &[&["Ann Able", "address=Univ A, Town"], &[
    "Bob Baker",
    "address=Univ B, City",
  ]];
  assert_reading(tex, Some(xml), &[
    "C Ann Able [] | line:names",
    "  a Univ A, Town | api:annotate:run",
    "C Bob Baker [] | line:names",
    "  a Univ B, City | api:annotate:run",
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
