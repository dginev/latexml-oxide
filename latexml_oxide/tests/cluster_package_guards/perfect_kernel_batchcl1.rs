//! Guards for CL1 (`agent_reports/2026-11-08_brief_CL1.md`; w56's report): under `LATEXML_FM_EMIT=2` the emitter reads on
//! past an accent's argument as TeX reads an undelimited one — one token, a control word included, or a balanced group
//! (`observer/writer/emit.rs` `past_what_it_set`). A name ending in ``\`\i`` lost its letter, "Chiriv`" (1709.03335).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

use super::perfect_kernel_batch84b::creators;

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// ``\`\i``, `\'{\i}`, `\^\j`, `\"\i`, `\o`, `\v{c}`, ``\`{\i}`` and ``\`{ab}`` last in a name: off and `=2` write every name
/// whole with its accent, each with its address, and no Warning (the base's `=2` wrote "Chiriv`", "Maˆ", "Fa¨" and two
/// content-check Warnings). One guard: the accent's argument is one code path, a control word its only new shape.
#[test]
fn an_accent_over_a_control_word_keeps_its_letter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/emit_an_accent_over_a_control_word_keeps_its_letter.tex"
  );
  let expected = [
    row(&["Rocco Chirivì", "address=Univ A"]),
    row(&["Ana Marí", "address=Univ B"]),
    row(&["Bo Maĵ", "address=Univ C"]),
    row(&["Lu Faï", "address=Univ D"]),
    row(&["Ole Hansø", "address=Univ E"]),
    row(&["Eva Petrič", "address=Univ F"]),
    row(&["Al Tiì", "address=Univ G"]),
    row(&["Di Niàb", "address=Univ H"]),
  ];
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
  }
}
