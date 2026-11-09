//! Guards for E1 (`agent_reports/2026-11-08_brief_E1.md`; w56's report): a Declaration's span runs to the close of the group
//! its call opened, as TeX reads an undelimited argument whole (`observer/field_notes.rs` `balanced_reach`). Under
//! `LATEXML_FM_EMIT=2` the emitter replayed icrc2009's `\IEEEauthorblockN{` cut from its argument: `readBalanced ran out`
//! Errors the document does not raise (1111.2718).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

use super::perfect_kernel_batch84b::creators;

/// A macro defining after it read its argument, the names in it (`\blockN{Bob Baker}`, a nested group in `\blockN{Cy {Dee}
/// Ewe}`), and one whose argument prints nothing (`\setx{unused}`): off and `=2` write the three names as personnames, keep
/// both definitions, and raise no Error or Warning (the base's `=2` raised one `readBalanced ran out`). One guard: the
/// declaration's reach is one code path.
#[test]
fn a_declaration_keeps_its_calls_argument() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/emit_a_declaration_keeps_its_calls_argument.tex"
  );
  let expected: Vec<Vec<String>> = ["Ann Able", "Bob Baker", "Cy Dee Ewe"]
    .iter()
    .map(|name| vec![name.to_string()])
    .collect();
  for emit in [0, 2] {
    let (log, xml, ()) = convert_files_with_setup_then(
      tex,
      &[],
      Some("ar5iv.sty"),
      move || {
        set_fm_shadow_override(None);
        set_fm_emit_override(Some(emit));
      },
      |_| (),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}\n{log}"
    );
    assert_eq!(creators(&xml), expected, "emit={emit}");
    assert!(xml.contains("Space 0pt, set."), "emit={emit}\n{xml}");
  }
}
