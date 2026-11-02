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

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error, `unclaimed` net Warnings and no
/// other, and that both give the same XML; returns it squashed, from its `<title>` to its end.
#[track_caller]
fn converted(tex: &str, unclaimed: usize) -> String {
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

/// A1: the class's affiliation list, printed only by its own `\@author` (authblk's code embedded in the class,
/// copernicus.cls:1290-1352), which the kernel's `\author` replaced: kept for the speculative run of the title page, the
/// list is found printed and typeset by no kept run, and is the note of the trailing nameless creator below the
/// authors (the block's, OXIDIZED_DESIGN_DIVERGENCES #159), its leading `\\` taken off. Witnesses 0712.1312,
/// 1401.4706, 1709.00722.
#[test]
fn an_affiliation_list_only_the_class_author_prints_is_a_note_of_the_block() {
  assert_eq!(
    converted(
      repro!("net_affiliation_list_printed_only_by_the_class_author_is_a_block_note"),
      1
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <creator role="author"> <note role="note" xml:id="id1"><sup xml:id="id1.1"><text font="italic" xml:id="id1.1.1">1</text></sup>Institute of Risk and Disaster Reduction, University College London</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A2: the class's title note, kept in a token register only its refused `{abstract}` begin code prints in an
/// `\insert\footins` (sig-alternate.cls:728-790, :1374-1398): the register is given its sentinel (`\tntoks={…}`), the
/// begin code is run after the title page, and the note is the sole author's (OXIDIZED_DESIGN_DIVERGENCES #159).
/// Witnesses 1206.6940, cs0405006.
#[test]
fn a_title_note_register_only_the_class_abstract_prints_is_a_note() {
  assert_eq!(
    converted(
      repro!("net_title_note_register_printed_only_by_the_class_abstract_is_a_note"),
      1
    ),
    r#"<title>A Title<sup xml:id="id1"><text font="italic" xml:id="id1.1">∗</text></sup></title> <creator role="author"> <personname>Ann Able</personname> <note role="note" xml:id="id2">Supported by the Danish Council for Independent Research.</note> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">An abstract.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// The net's exclusions: a store the class's `\@author` prints that only repeats the authors' names (its words all the
/// creators') is no note, nor rows of the author block when K11's off-table survey takes it (`hand_offtable_stores`); one still at its class default (a letterhead), one the class's `\AtBeginDocument` set, and
/// one printed only in a running head (`\markboth`) are no candidates — no note and no Warning. A store that holds
/// words of its own, set by the document, is one (the same class, `\names` given an affiliation).
#[test]
fn a_store_the_frontmatter_holds_or_the_class_owns_is_no_note() {
  let tex = repro!("net_store_the_frontmatter_holds_or_the_class_owns_is_no_note");
  let plain = r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  assert_eq!(converted(tex, 0), plain);
  // (the same store made by a pure one-argument setter is K11's off-table store, printed beside the names by the class's
  // `\@author`: no rows of the author block either)
  let pure = tex.replace(r"\gdef\@names{\ignorespaces#1}", r"\gdef\@names{#1}");
  assert_ne!(pure, tex);
  assert_eq!(converted(&pure, 0), plain);
  // (the same store, holding text no field holds, is a note: the exclusions above are what kept it out)
  let worded = tex.replace(
    r"\names{Ann Able and Bob Baker}",
    r"\names{Ann Able and Bob Baker\\Lunar Geology Institute, Tycho}",
  );
  assert_ne!(worded, tex);
  assert_eq!(
    converted(&worded, 1),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <creator role="author"> <note role="note" xml:id="id1">Lunar Geology Institute, Tycho</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}
