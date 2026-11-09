//! Guards for CL6 (`agent_reports/2026-11-09_brief_CL6.md`, classes C1 and C2 of CL5's `=2` content losses): a document
//! contact command that reads more required arguments than the rehearsal's binding keeps its meaning
//! (`rehearsal.rs` `binding_required_arguments`). pnastwo's `\affil#1#2`, read as the one-argument affiliation, had the
//! emitter's item `\affil{1}` read past its end and `\gdef` `\affil1` to `\let`, which under `\@maketitle` ate the
//! title-page deposit's closing braces: a group left open after `\maketitle` (every body `\footnote` in it a `\thanks`,
//! 1312.3323), the deposit dropped for its error with the `\contributor` line in it (1404.2960, 0903.4054, 0907.3600).
//! The class is a cut of pnastwo.cls (`cl6_pnas_affil.cls`; pnastwo is not in TeX Live). Each guard off and at `=2`;
//! at `=2` the block is written whole (NE2b's counter-move decline: `\affil` steps the footnote counter for its mark),
//! with that one Warning.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz3::in_a_directory};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name
    ))
  };
}

const CLASS: (&str, &str) = ("cl6_pnas_affil.cls", repro!("cl6_pnas_affil.cls"));

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// The paragraphs' text of the document's body (tags stripped, whitespace collapsed), in order.
fn paragraphs(xml: &str) -> Vec<String> {
  xml
    .match_indices("<para ")
    .map(|(start, _)| {
      let end = start + xml[start..].find("</para>").expect("closed para");
      let mut text = String::new();
      let mut in_tag = false;
      for c in xml[start..end].chars() {
        match c {
          '<' => in_tag = true,
          '>' => in_tag = false,
          _ if !in_tag => text.push(c),
          _ => {},
        }
      }
      text.split_whitespace().collect::<Vec<_>>().join(" ")
    })
    .collect()
}

/// The group repro: `\maketitle` leaves the group level at 1 and the body's `\footnote` is a footnote in the body, the
/// title-page deposit (the printed affiliation, the default contributor line) kept — at `=2` the level was 2, the note
/// a `<pubnote>` and the deposit dropped, with an "open groups" Warning.
#[test]
fn a_two_argument_affil_keeps_its_meaning_and_closes_its_groups() {
  let names = vec![row(&["Ann Able ∗"])];
  let (log, xml) = in_a_directory(repro!("cl6_pnas_affil_group.tex"), &[CLASS], &[]);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("GROUPLEVEL=1"), "{log}");
  assert!(!log.contains("open groups"), "{log}");
  assert_eq!(creators(&xml), names);
  assert!(!xml.contains("<pubnote"), "{xml}");
  assert_eq!(
    paragraphs(&xml),
    [
      "∗Univ X",
      "Submitted to the Proceedings",
      "Body 1 1 footnote 1 BODYNOTE alpha. text."
    ],
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p3.1">Body<note mark="1" role="footnote" xml:id="footnote1">"#),
    "{xml}"
  );
}

/// The `\contributor` line `\@maketitle` prints is in the title-page deposit — the emitter dropped the deposit once, the
/// line counted `unclaimed`.
#[test]
fn the_contributor_line_is_in_the_title_page_deposit() {
  let names = vec![row(&["Ann Able ∗ and Bob Baker †"])];
  let (log, xml) = in_a_directory(repro!("cl6_pnas_contributor.tex"), &[CLASS], &[]);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(!log.contains("unclaimed"), "{log}");
  assert_eq!(creators(&xml), names);
  assert_eq!(
    paragraphs(&xml),
    ["∗Univ X,†Univ Y", "Submitted by Carl Contributor", "Text."],
    "{xml}"
  );
}
