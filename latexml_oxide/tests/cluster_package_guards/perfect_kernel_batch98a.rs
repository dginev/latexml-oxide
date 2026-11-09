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
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
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
