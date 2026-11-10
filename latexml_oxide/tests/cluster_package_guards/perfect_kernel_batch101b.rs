//! Red/green guards for batch 101b: an author block's address line carrying a `\thanks` or `\footnote` is still an
//! address line, its notes the document's (`defs/frontmatter_api/email_lines.rs` `address_line_notes`, the one conversion
//! `frontmatter/observer/emitter/calls.rs` `document_note_call`; OXIDIZED_DESIGN_DIVERGENCES #52(j)). Read whole, the note's text is
//! no address, so the line was an affiliation: the last name's in a marked block, every name's in an unmarked one.
//! Witness 2605.23553 (html_feedback #6987); 2304.05485, 1803.07937, 2603.09888, 2608.24641.
//!
//! Each guard converts its repros under `tools/perfect_kernel/repros/sectioning-frontmatter/` in process and asserts
//! no Error and no Warning, every creator whole, in order, and the document's pubnotes whole. One guard per code path:
//! the kernel's marked block (a `\thanks`, a `\footnote`, a labelled e-mail line), its unmarked block, and a marked
//! affiliation list's address run (`entry_calls`), with the frontmatter observer on and off — its writer read that
//! line right already, the token reader now agrees.
use latexml::util::test::convert_with_setup_then;

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Converts `tex` in process, checks it has no Error and no Warning, and returns its creators, then its pubnotes, each whitespace run one space and without
/// `xml:id`s.
#[track_caller]
fn creators_and_pubnotes(tex: &str) -> (String, Vec<String>) {
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| ());
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let mut without_ids = String::with_capacity(xml.len());
  let mut rest = xml.as_str();
  while let Some(at) = rest.find(" xml:id=\"") {
    without_ids.push_str(&rest[..at]);
    let value = &rest[at + " xml:id=\"".len()..];
    rest = &value[value.find('"').expect("a closed xml:id") + 1..];
  }
  without_ids.push_str(rest);
  let xml = without_ids;
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  let pubnotes = xml
    .match_indices("<pubnote")
    .map(|(at, _)| {
      let close = xml[at..].find("</pubnote>").expect("a closed pubnote") + "</pubnote>".len();
      xml[at..at + close].to_string()
    })
    .collect();
  (xml[start..end].to_string(), pubnotes)
}

/// The four creators of the marked block, each its affiliation and the address that spells its name.
const MARKED_CREATORS: &str = r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ X, Italy</contact> <contact name="Email: " role="email"><text font="typewriter">ann.able@x.it</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ Y, Italy</contact> <contact name="Email: " role="email"><text font="typewriter">bob.baker@y.it</text></contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation">Univ X, Italy</contact> <contact name="Email: " role="email"><text font="typewriter">cy.cole@x.it</text></contact> </creator> <creator before=" " role="author"> <personname>Dan Dorn</personname> <contact name="Affiliation: " role="affiliation">Univ Y, Italy</contact> <contact name="Email: " role="email"><text font="typewriter">dan.dorn@y.it</text></contact> </creator>"#;

/// The marked block (IEEEtran, 2605.23553's shape): the second address line, carrying the submission `\thanks`, gives
/// Cy Cole and Dan Dorn their e-mails (before: one affiliation "cy.cole@x.it, dan.dorn@y.it" of Dan Dorn), the note a
/// `thanks` pubnote; a `\footnote` the same, a role-less pubnote; a labelled line (`E-mail: …\thanks{…}`) read as a
/// labelled e-mail line (before: welded into Bob Baker's affiliation).
#[test]
fn an_address_line_carrying_a_note_in_a_marked_block_is_one() {
  assert_eq!(
    creators_and_pubnotes(repro!("address_line_note_marked_block")),
    (MARKED_CREATORS.to_string(), vec![
      r#"<pubnote role="thanks">Submitted somewhere.</pubnote>"#.to_string()
    ])
  );
  assert_eq!(
    creators_and_pubnotes(repro!("address_line_footnote_marked_block")),
    (MARKED_CREATORS.to_string(), vec![
      "<pubnote>Submitted somewhere.</pubnote>".to_string()
    ])
  );
  assert_eq!(
    creators_and_pubnotes(repro!("address_line_note_labelled_marked_block")),
    (
      r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ X, Italy</contact> <contact name="E-mail: " role="email"><text font="typewriter">ann.able@x.it</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ Y, Italy</contact> <contact name="E-mail: " role="email"><text font="typewriter">bob.baker@y.it</text></contact> </creator>"#
        .to_string(),
      vec![r#"<pubnote role="thanks">Submitted somewhere.</pubnote>"#.to_string()]
    )
  );
}

/// The unmarked article block: the address line under the shared institution gives each name its e-mail (before: one
/// affiliation "ann.able@x.it, bob.baker@y.it" of both), the note a `thanks` pubnote.
#[test]
fn an_address_line_carrying_a_note_in_an_unmarked_block_is_one() {
  assert_eq!(
    creators_and_pubnotes(repro!("address_line_note_unmarked_block")),
    (
      r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ X, Italy</contact> <contact name="Email: " role="email"><text font="typewriter">ann.able@x.it</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ X, Italy</contact> <contact name="Email: " role="email"><text font="typewriter">bob.baker@y.it</text></contact> </creator>"#
        .to_string(),
      vec![r#"<pubnote role="thanks">Submitted somewhere.</pubnote>"#.to_string()]
    )
  );
}
