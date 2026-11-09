//! Guards for batch 95e: the written-out e-mails of one `\email` call printing several addresses are given back whole
//! for the writer's name re-part (`writer::give_back_listed` (deleted)), as today's holds them — the one contact on the call's
//! author —, so a names call the reading parts is re-parted (OXIDIZED_DESIGN_DIVERGENCES #928) though 95c decides the
//! call address by address (#938 G3); the plan writes them out after it, and where it cannot, each call stays one
//! contact (the fail-safe: an e-mail split never declines a paper's names). OXIDIZED_DESIGN_DIVERGENCES #938 (95e). Witness
//! 2109.07363 (bundle int124b's A/B: one creator, `merged_creators`). Each repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` compiles clean with pdflatex (TL 2025); each guard asserts the
//! reading's tuples and the writer's outcome (in process and the binary, eager and
//! `--streaming`) and the default mode's creators as whole elements (in process and the binary, eager and
//! `--streaming`), every conversion with no Error and no Warning.

use serde_json::Value;

use super::perfect_kernel_batch92a::assert_reading;

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// The fail-safe: the write-out cannot copy today's contact (an `\href` around the first address: no one text leaf),
/// so the call stays one contact on its author, its link kept, and the names are still re-parted — the plan planned
/// again with the call whole (the S2b writer, deleted in DEL4). Distinct from the guard above: the plan's own
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
}
