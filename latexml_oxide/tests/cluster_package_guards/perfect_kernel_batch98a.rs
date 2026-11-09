//! Guards for batch 98a (review rev126a's blockers on bundle int126a, the last fix round of 97e and 97f): a
//! collaboration heading a printed separator parts from the first name is an entry of the names list, as the old reader
//! and Perl read it — the observer writer's "a name naming no person" decline reads the separator after the heading
//! too, not only the one before it (B1, 2206.00684); and a control word inside an `\altaffilmark` list is no label of the
//! observer's request, as it is none of the binding's key (B3, astro-ph0105133: one helper,
//! `relocate::mark_spelling_printed`). Each guard converts its repros under the shadow switch in process and with the
//! binary, eager and `--streaming` (`perfect_kernel_batch92a::assert_reading`), and asserts the XML's whole creator set
//! with every contact, the reading's tuples, the writer's outcome, the default mode writing the same, and no Error or
//! Warning. Every repro compiles clean with pdflatex (TL 2025; svjour with 0710.5369's svjour.cls and svepj.clo).

use super::{
  perfect_kernel_batch92a::assert_reading, perfect_kernel_batch95b::default_mode_creators,
  perfect_kernel_batcht1::red_at_two,
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// llncs `\author{XYZ Collaboration\thanks{…}: Ann Able\inst{1} \and Bob Baker\inst{2}}` prints "XYZ Collaboration⋆ :
/// Ann Able1 and Bob Baker2": the colon parts the heading from the first name. Since 103b the heading is Ann Able's
/// `contact role="collaboration"` and the writer declines (the outcome this guard is named for); 98a read it as an entry
/// of the list (97b's and 97c's reading, 2206.00684's GRAVITY+ Collaboration; 2407.21626's `\and The ABC Collaboration\thanks{…}:
/// Fay Fox` is parted by the separator before it). int126a declined the re-part — its "a name naming no person" read
/// only the separator before the heading, which the first piece has none of — and the old reader wrote "XYZ
/// Collaboration: Ann Able" as one name. Distinct from its sibling on the same decline: svjour's `{The XYZ
/// Collaboration}\\[2ex]Ann Able\inst{1}`, parted by the line alone (1805.08173), still declines, today's one creator
/// kept (the bracketed remark `(XYZ Collaboration)\\ \\`: `perfect_kernel_batch97e::a_collaboration_heading_is_no_creator`).
/// Since 103b the llncs split reads the heading itself (`split_author_calls`): Ann Able's collaboration contact, the
/// heading's note a thanks pubnote, the reading's entry declined by the writer as a kernel colon heading is (OD #946;
/// charter §4: a collaboration is a contact on its names, no creator).
#[test]
fn a_collaboration_heading_a_separator_parts_is_a_contact_the_writer_declines() {
  let tex = repro!("llncs_collaboration_colon_heading_keeps_first_name.tex");
  if red_at_two(
    r#"makes "XYZ Collaboration" its own creator instead of each name's collaboration contact"#,
  ) {
    return;
  }
  let (xml, _, notes, writer) = assert_reading(
    tex,
    Some(&[
      &[
        "Ann Able",
        "collaboration=XYZ Collaboration",
        "affiliation=Univ A",
      ],
      &["Bob Baker", "affiliation=Univ B"],
    ]),
    &[
      "C XYZ Collaboration [] | line:names",
      "  n Corresponding author: Ann Able | note:adjacent",
      "C Ann Able [1] | line:names sep:word",
      "  a Univ A | api:labelseq",
      "C Bob Baker [2] | line:names",
      "  a Univ B | api:labelseq",
    ],
  );
  // (today's reader reads the heading as llncs's split reads it since 103b — the collaboration of the name after it,
  // its note a thanks pubnote, as the kernel's author line does — so the writer declines as for a kernel colon
  // heading, OXIDIZED_DESIGN_DIVERGENCES #946: today's XML kept)
  assert_eq!(
    writer, "declined: the creators' names",
    "the writer's outcome"
  );
  assert_eq!(
    notes,
    ["Corresponding author: Ann Able"],
    "the heading's note"
  );
  assert_eq!(default_mode_creators(tex), xml, "the default mode");

  let tex = repro!("svjour_collaboration_line_heading_is_no_creator.tex");
  let (xml, _, _, writer) = assert_reading(
    tex,
    Some(&[
      &[
        "The XYZ Collaboration Ann Able",
        "affiliation=Univ A, Town A",
      ],
      &["Bob Baker", "affiliation=Univ B, Town B"],
    ]),
    &[
      "C The XYZ Collaboration [] | line:names",
      "C Ann Able [1] | line:names",
      "  a Univ A, Town A | api:labelseq",
      "C Bob Baker [2] | line:names",
      "  a Univ B, Town B | api:labelseq",
    ],
  );
  assert!(
    writer.as_str().is_some_and(|w| w.starts_with("declined: ")),
    "the writer's outcome: {writer}"
  );
  assert_eq!(default_mode_creators(tex), xml, "the default mode");
}

/// emulateapj `\author{Ann Able\altaffilmark{1,\hbox{ }2}, Bob Baker\altaffilmark{2}}` prints "Ann Able1, 2": the
/// observer's request labels are the items as they print, their control words and braces dropped as the binding's
/// key drops them (`relocate::mark_key`, one helper `mark_spelling_printed`), so Ann Able requests `altaffil:1` and
/// `altaffil:2` and holds both lines (97f's output; astro-ph0105133's Yu-Qing Lou and Zuhui Fan). int126a split the
/// item at its space into `altaffil:\hbox` and a bare `2`, and Ann Able lost Uni Two.
#[test]
fn a_control_word_in_an_altaffilmark_list_is_no_label() {
  let tex = repro!("aastex_altaffilmark_hbox_spaced_marks.tex");
  let (xml, ..) = assert_reading(
    tex,
    Some(&[
      &["Ann Able", "affiliation=Uni One", "affiliation=Uni Two"],
      &["Bob Baker", "affiliation=Uni Two"],
    ]),
    &[
      "C Ann Able [altaffil:1,altaffil:2] | line:names",
      "  a Uni One | api:label ruling1:request",
      "  a Uni Two | api:label ruling1:request",
      "C Bob Baker [altaffil:2] | line:names sep:comma",
      "  a Uni Two | api:label ruling1:request",
    ],
  );
  assert_eq!(default_mode_creators(tex), xml, "the default mode");
}
