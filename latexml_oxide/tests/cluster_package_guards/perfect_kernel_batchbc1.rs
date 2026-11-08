//! Guards for BC1, the one-pass fold's first step (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`; OXIDIZED_DESIGN_DIVERGENCES
//! #980): at `LATEXML_FM_EMIT=2` the emitter writes the author block's own creator (OXIDIZED_DESIGN #159), which relocate's
//! orphan-sharing branch made before.

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
};

/// A block's e-mail line no name spells (`label=addresses:block`): the nameless creator below the names holds it, written by
/// the emitter (`writer::emit::block_creator`) and found by the label as Perl places any annotation, uncounted among the names
/// (no `_num`: Bob Baker's separator stays the last name's). Relocate's orphan-sharing branch does not run for an emitted paper, so without
/// the emitter's creator the address would be orphaned: a Warning and the e-mail lost. Witnesses 1012.4847, 1106.3045 (byte
/// identical before and after on int149a `=2`). The creators' XML is the switch-off one.
#[test]
fn the_blocks_creator_is_the_emitters() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/emit_the_blocks_creator_is_the_emitters.tex"
  );
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want = [
    vec!["Ann Able".to_string(), "affiliation=Univ A".to_string()],
    vec!["Bob Baker".to_string(), "affiliation=Univ A".to_string()],
    vec!["email=lab@a.org".to_string()],
  ];
  assert_eq!(creators(&xml), want, "{xml}");
  assert_eq!(creators_xml(&xml), creators_xml(&off));
  assert!(
    creators_xml(&xml)[2].starts_with(r#"<creator role="author">"#),
    "{xml}"
  );
}
