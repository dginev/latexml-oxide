//! Red/green guards for batch 98c: the unclaimed-text net (FIDELITY_CHARTER §4 "No printed frontmatter word is lost";
//! OXIDIZED_DESIGN_DIVERGENCES #953): a raw class's store that only its own title-page code prints, which no
//! frontmatter owner claims and no kept title-page run typeset, is written as a note of the author block with one
//! Warning (`frontmatter:unclaimed`), where its lines hold words the frontmatter does not
//! (latexml_engine/src/frontmatter_stores.rs `net_candidates`, `hand_net_stores`; sect05.rs
//! `keep_class_author_reader`; base_utilities/frontmatter/insert.rs `keep_unclaimed_net_notes`). Three code paths: a
//! macro store printed by the class's own `\@author` (A1: 0712.1312, 1401.4706, 1709.00722), a token register printed
//! by the class's refused `{abstract}` begin code (A2: 1206.6940, cs0405006; 2606.24738), and the candidate rule's
//! exclusions (the names repeated, a class default, a begin-document hook's value, a running head). Each guard
//! converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and
//! `--streaming`, and asserts the frontmatter it makes whole.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch89a::cli_conversion,
};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error, `unclaimed` net Warnings and no
/// other, and that both give the same XML; returns it squashed, from its `<title>` to its end.
#[track_caller]
pub(super) fn converted(tex: &str, unclaimed: usize) -> String {
  let mut xmls = Vec::new();
  for streaming in [false, true] {
    let (log, xml) = cli_conversion(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, unclaimed),
      "{log}"
    );
    assert_eq!(
      log.matches("Warning:frontmatter:unclaimed").count(),
      unclaimed,
      "{log}"
    );
    xmls.push(xml);
  }
  assert_eq!(xmls[0], xmls[1]);
  let xml = squash(&xmls[0]);
  let start = xml.find("<title>").unwrap_or(0);
  let end = xml[start..]
    .find("</document>")
    .map_or(xml.len(), |end| start + end);
  xml[start..end].trim_end().to_string()
}
