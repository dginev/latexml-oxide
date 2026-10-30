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
  let xml: &[&[&str]] = &[
    &[
      "Ann Able 1",
      "address=1 Univ A, Town, Country",
      "email=ann@a.edu",
    ],
    &["Bob Baker 1", "address=1 Univ A, Town, Country"],
    &["Cy Cole 2", "address=2 Univ B, City, Country"],
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
  let xml: &[&[&str]] = &[
    &[
      "Ann Able 1,2",
      "address=1 Univ A, Town",
      "address=2 Univ B, City",
    ],
    &["Bob Baker 3", "address=3 Lab C, Place"],
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
}
