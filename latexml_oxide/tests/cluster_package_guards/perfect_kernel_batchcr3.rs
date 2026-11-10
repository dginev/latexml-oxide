//! Red/green guards for batch CR3: three content-residual items — a date set inside an `\author` nested in `\title`
//! counted, a recovered call's run of several creators placed without a joined label, and the shipped imsart copies read
//! through their binding. OXIDIZED_DESIGN_DIVERGENCES #985, #986.
use super::perfect_kernel_batchcr2::{convert_clean, elements};

/// CR3 item 2 (review revCR2b finding 1 and P1): REVTeX 3's minipage `\address` (abstract and `\pacs` only) after
/// `\author{Ann Able and Bob Baker}`, then `\address{Univ A, Budapest}`. The recovered call's run is two creators, so it
/// goes out unlabelled (no `label=author:1,author:2` the insertion cannot find: no orphaned-annotation Warning), and its
/// empty entry records the run, so Univ A is both authors', as the old driver. Distinct from cr2's
/// `revtex3_minipage_address_keeps_contact_order` (one shared item label). Repro
/// sectioning-frontmatter/revtex3_minipage_address_first_joins_run_labels.
#[test]
fn revtex3_minipage_address_first_one_run() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex3_minipage_address_first_joins_run_labels.tex"
  );
  let xml = convert_clean(tex);
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ A, Budapest</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ A, Budapest</contact>\n  </creator>",
  ]);
  assert_eq!(elements(&xml, "abstract"), vec![
    "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\">\n    <p xml:id=\"abstract1.1\">We study a zebraword model.</p>\n  </abstract>",
  ]);
  assert_eq!(elements(&xml, "classification"), vec![
    "<classification name=\"PACS:\u{a0}\" scheme=\"pacs\">PACS numbers: 05.40.+j</classification>",
  ]);
}
