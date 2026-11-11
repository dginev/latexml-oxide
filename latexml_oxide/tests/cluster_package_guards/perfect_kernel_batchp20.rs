//! Guards for P20, Option C step P2-0 (OXIDIZED_DESIGN_DIVERGENCES #995): the two mechanisms the P2 reroutes call, which
//! no binding calls yet.
//!
//! - The binding-callable separator query (`observer/separators.rs` `argument_separators`, its TeX front-end
//!   `\lx@fm@separators{<tokens>}`): the `\and`-family separators an argument holds at its own level, as the rehearsal
//!   digests it, each with the meaning of the token that printed it (`kernel`, `binding`, `raw`). Three rows, one per
//!   meaning: the kernel's (a literal `\and`, `\And`/`\AND`, a macro-held one, a `\let` copy, a `\csname`; none inside a
//!   box's tabular, none for the word), a raw class's (a word, a `&`: trap T8, the rehearsal's `Sep` alone says nothing of
//!   the meaning), a binding's (aas_support's `\and`).
//! - The emitter's cut of a numbered contact call (`emit::labelseq_call_cut`): a `labelseq=` call the reading cut at its
//!   `\and`s (`resolver::api::labelseq_pieces`) written as one call per piece, by its tokens (a literal `\and`) or by its
//!   text, one counted Warning a piece (a macro holding the entries and the `\and`s).
//!
//! Each converts its repro, a document calling the query or the API itself, and asserts the exact Error and Warning counts
//! and the whole answer: every line's list, every creator's name and contacts in order (the literal row's creators whole).

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
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

/// The text of each `<p>` of `xml`, in order.
fn paragraphs(xml: &str) -> Vec<String> {
  xml
    .match_indices("<p ")
    .filter_map(|(at, _)| {
      let open = at + xml[at..].find('>')? + 1;
      let close = open + xml[open..].find("</p>")?;
      Some(xml[open..close].trim().to_string())
    })
    .collect()
}

/// `tex` converted clean, its paragraphs.
fn answers(tex: &str) -> Vec<String> {
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  paragraphs(&xml)
}

/// The kernel separator, however the argument spells it: literal, `\And`/`\AND`, held in a macro of the document's, a
/// `\let` copy, a `\csname and\endcsname`. One inside a `\parbox`'s tabular is the box's (nest 1), and the word "and"
/// is none.
#[test]
fn the_separator_query_reads_the_kernel_meanings() {
  assert_eq!(answers(repro!("separators_query_kernel_meanings")), [
    "L1[kernel]",
    "L2[kernel,kernel]",
    "L3[kernel]",
    "L4[kernel]",
    "L5[kernel]",
    "L6[]",
    "L7[]"
  ]);
}

/// A class's `\and` printing the word or a `&` still digests to the rehearsal's separator event (`ALWAYS_BOUND`), and the
/// query says it is no kernel separator: `raw`, spelled, copied or held in a macro (trap T8). The kernel's `\and` after
/// the group is `kernel` again.
#[test]
fn the_separator_query_reads_a_class_meaning_as_raw() {
  assert_eq!(answers(repro!("separators_query_class_meanings")), [
    "W1[raw]",
    "W2[raw]",
    "W3[raw]",
    "A1[raw,raw]",
    "K1[kernel]"
  ]);
}

/// A binding's own `\and` (aas_support's, under aastex701) is `binding`: what that binding says it means.
#[test]
fn the_separator_query_reads_a_binding_meaning() {
  assert_eq!(answers(repro!("separators_query_binding_meaning")), [
    "B1[binding]"
  ]);
}

/// A `labelseq=affiliation` call holding `Univ X \and Univ Y \and Univ Z` is written as three calls, each its piece alone
/// under the reading's label, the call's keyvals kept: each author holds the institute their mark requests, as the
/// class's numbered list prints. By tokens (a literal `\and`: no Warning) and by text (a macro holding the entries and
/// the `\and`s: one Warning a piece).
#[test]
fn a_numbered_contact_call_is_written_a_call_per_piece() {
  let (log, xml) = run(repro!("labelseq_contact_call_cut_at_its_and"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators_xml(&xml), [
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ X</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Y</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Coe</personname>\n    <contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z</contact>\n  </creator>",
  ]);
  let (log, xml) = run(repro!("labelseq_contact_call_cut_at_a_macro_held_and"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_eq!(creators(&xml), [
    vec!["Ann Able".to_string(), "affiliation=Univ X".to_string()],
    vec!["Bob Baker".to_string(), "affiliation=Univ Y".to_string()],
  ]);
}
