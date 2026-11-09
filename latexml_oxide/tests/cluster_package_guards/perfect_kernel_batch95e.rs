//! Guards for batch 95e: the written-out e-mails of one `\email` call printing several addresses are given back whole
//! for the writer's name re-part (`writer::give_back_listed`), as today's holds them — the one contact on the call's
//! author —, so a names call the reading parts is re-parted (OXIDIZED_DESIGN_DIVERGENCES #928) though 95c decides the
//! call address by address (#938 G3); the plan writes them out after it, and where it cannot, each call stays one
//! contact (the fail-safe: an e-mail split never declines a paper's names). OXIDIZED_DESIGN_DIVERGENCES #938 (95e). Witness
//! 2109.07363 (bundle int124b's A/B: one creator, `merged_creators`). Each repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` compiles clean with pdflatex (TL 2025); each guard asserts the
//! reading's tuples and the writer's outcome under the shadow switch (in process and the binary, eager and
//! `--streaming`) and the default mode's creators as whole elements (in process and the binary, eager and
//! `--streaming`), every conversion with no Error and no Warning.

use serde_json::Value;

use super::{
  perfect_kernel_batch92a::assert_reading, perfect_kernel_batch95c::assert_elements,
  perfect_kernel_batcht1::red_at_two,
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// The re-part reads the call whole: `\author{Ann Able\textsuperscript{1} $\&$ Bob Baker\textsuperscript{2}}` with
/// `\email{ann@x.org,\:bob@y.org}` (2109.07363's shape) is two creators, each its address — int124b's writer declined
/// the re-part ("an item with nothing to write it from") and left one creator; and its three-name sibling with three
/// addresses (the witness's count; two `\address`es under three names stay the call's author's, as int123a's).
#[test]
fn a_names_call_with_a_several_address_email_is_reparted() {
  let tex = repro!("amsart_names_call_with_a_several_address_email.tex");
  if red_at_two(r#"a false plain-text Warning: ann@x.org loses its markup"#) {
    return;
  }
  let reading = assert_reading(tex, None, &[
    "C Ann Able [1] | line:names",
    "  a 1. Univ X | api:annotate:last owner:upfront-grid",
    "  c email ann@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "C Bob Baker [2] | line:names sep:word",
    "  a 2. Univ Y | api:annotate:last",
    "  c email bob@y.org | api:annotate:last ruling3:written-out owner:spelled",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_elements(tex, &[
    "<creator role=\"author\"><personname>Ann Able<sup>1</sup></personname><contact name=\"Address:\u{a0}\" \
     role=\"address\"><M/> Univ X</contact><contact name=\"Email address: \" role=\"email\"><text \
     font=\"typewriter\">ann@x.org</text></contact></creator>",
    "<creator role=\"author\"><personname>Bob Baker<sup>2</sup></personname><contact name=\"Address:\u{a0}\" \
     role=\"address\"><M/> Univ Y</contact><contact name=\"Email address: \" role=\"email\"><text \
     font=\"typewriter\">bob@y.org</text></contact></creator>",
  ]);
  let tex = repro!("amsart_names_call_of_three_with_a_several_address_email.tex");
  let reading = assert_reading(tex, None, &[
    "C Ann Able [1] | line:names",
    "  c email ann@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "C Bob Baker [1] | line:names sep:word",
    "  c email bob@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "C Cat Cole [2] | line:names sep:word",
    "  a 1. Univ X | api:annotate:last",
    "  a 2. Univ Y | api:annotate:last",
    "  c email cat@y.org | api:annotate:last ruling3:written-out owner:spelled",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_elements(tex, &[
    "<creator role=\"author\"><personname>Ann Able<sup>1</sup></personname><contact name=\"Email address: \" \
     role=\"email\"><text font=\"typewriter\">ann@x.org</text></contact></creator>",
    "<creator role=\"author\"><personname>Bob Baker<sup>1</sup></personname><contact name=\"Email address: \" \
     role=\"email\"><text font=\"typewriter\">bob@x.org</text></contact></creator>",
    "<creator role=\"author\"><personname>Cat Cole<sup>2</sup></personname><contact name=\"Address:\u{a0}\" \
     role=\"address\"><M/> Univ X</contact><contact name=\"Address:\u{a0}\" role=\"address\"><M/> Univ \
     Y</contact><contact name=\"Email address: \" role=\"email\"><text \
     font=\"typewriter\">cat@y.org</text></contact></creator>",
  ]);
}

/// The fail-safe: the write-out cannot copy today's contact (an `\href` around the first address: no one text leaf),
/// so the call stays one contact on its author, its link kept, and the names are still re-parted — the plan planned
/// again with the call whole (`writer::write_observed_frontmatter`). Distinct from the guard above: the plan's own
/// decline is the path.
#[test]
fn a_several_address_email_the_plan_cannot_write_out_stays_whole() {
  let tex = repro!("amsart_names_call_with_a_several_address_email_kept_whole.tex");
  let reading = assert_reading(tex, None, &[
    "C Ann Able [] | line:names",
    "  c email ann@x.org | api:annotate:last ruling3:written-out owner:spelled",
    "C Bob Baker [] | line:names sep:word",
    "  a Univ X | api:annotate:last",
    "  c email bob@y.org | api:annotate:last ruling3:written-out owner:spelled",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  if red_at_two(
    r#"unverified: gives ann@x.org to Ann Able where the off reading keeps the email whole with Bob Baker"#,
  ) {
    return;
  }
  assert_elements(tex, &[
    "<creator role=\"author\"><personname>Ann Able</personname></creator>",
    "<creator role=\"author\"><personname>Bob Baker</personname><contact name=\"Address:\u{a0}\" \
     role=\"address\">Univ X</contact><contact name=\"Email address: \" role=\"email\"><ref class=\"ltx_href\" \
     font=\"typewriter\" href=\"mailto:ann@x.org\">ann@x.org</ref><text \
     font=\"typewriter\">,\u{2005}bob@y.org</text></contact></creator>",
  ]);
}
