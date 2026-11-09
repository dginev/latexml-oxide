//! Guards for CL2b (`agent_reports/2026-11-08_brief_CL2b.md`; CL2's report): a binding's list setter called inside an author
//! block (`observer/ledger.rs` `list_setter_called`, `AuthorBlock::list_setter`) — the emitter stays out of the paper
//! (`observer/writer/emit.rs` `stays_out`), each block written whole and its queued entries placed as off places them.
//! cond-mat0605195's `\and{Cy Coe\inst{1}\institute{…}}` had its marks glued to the names and "Lab Y" lost at `=2`.

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

use super::perfect_kernel_batch84b::creators;

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// svjour's `\institute{Univ X \and Lab Y}` inside the last author's group: off and `=2` give each author the institute
/// its `\inst` mark names, no mark in a name, no orphan; `=2` counts its staying out by one Warning. One guard: the list
/// setter in a block is one code path (the ledger's fact, the emitter's reason).
#[test]
fn a_list_setter_in_an_author_block_declines_the_emitter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/emit_a_list_setter_in_an_author_block_declines.tex"
  );
  let expected = [
    row(&["Ann Able", "affiliation=Univ X, City"]),
    row(&["Bob Baker", "affiliation=Lab Y, Town"]),
    row(&["Cy Coe", "affiliation=Univ X, City"]),
  ];
  for (emit, warnings) in [(0, 0), (2, 1)] {
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
      (0, warnings),
      "emit={emit}\n{log}"
    );
    assert!(
      !log.contains("Orphaned frontmatter annotation"),
      "emit={emit}\n{log}"
    );
    if emit == 2 {
      assert!(
        log
          .contains("can't read an author block (a binding's list setter called inside the block)"),
        "{log}"
      );
    }
    assert_eq!(creators(&xml), expected, "emit={emit}");
  }
}
