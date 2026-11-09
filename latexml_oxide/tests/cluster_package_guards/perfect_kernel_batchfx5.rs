//! Guards for FX5, an entry's e-mail under `LATEXML_FM_EMIT=2` (OXIDIZED_DESIGN_DIVERGENCES #980) when the entry is an affiliation
//! its owner already holds from another source: the e-mail kept as its own contact
//! (`an_institute_email_under_a_repeated_affiliation_is_kept`).

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

/// svjour3's `\institute{Ann One \at First Inst, UK\\ \email{ann@one.uk} \and …}` under an `\author` line printing the same
/// affiliation by a mark (2111.06447): the emitter writes each owner's affiliation text once, so the institute's entry, the
/// author line's affiliation again, is not written — and the e-mail inside it, which the entry was to take along, is its own
/// contact item (`items_to_emit`'s `separate`). On FX5base `=2` the address was lost, the content check counting it; off held
/// it. The one Warning left is the content check's count of "First Inst, UK", printed twice and held once. Distinct from
/// `perfect_kernel_batch98b::svjour3_text_before_at_is_kept` (the text before `\at`, no repeated entry).
#[test]
fn an_institute_email_under_a_repeated_affiliation_is_kept() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/svjour3_institute_email_under_a_repeated_affiliation.tex"
  );
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  assert!(off.contains(r#"<contact name="E-mail: " role="email">ann@one.uk</contact>"#));
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("missing from the placed frontmatter: First Inst UK."),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    vec![
      "Ann One 1",
      "affiliation=First Inst, UK",
      "email=ann@one.uk"
    ],
    vec![
      "Bob Two 2",
      "affiliation=Second Inst, France",
      "affiliation=Third Lab, France"
    ],
  ]);
  assert_eq!(xml.matches("ann@one.uk").count(), 1);
}
