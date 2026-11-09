//! Guards for NE1 (`agent_reports/2026-11-09_NE1_report.md`): two `LATEXML_FM_EMIT=2`-only Errors the switch off never
//! raises. (1) A name or an item the reading joins across an alignment's cells: the emitter declines the block
//! (`emit::emit_commands`' `across_cells`, from the digested `Layout{role="cell"}` events: `repart::joined_across_cells`,
//! `repart::crosses_columns`), which is then Perl's whole-block creator, as off writes it, with a counted Warning; the
//! backstop `Field::content_cut` refuses a run holding a stray `&` (`plan::holds_stray_alignment`). (2) A line of an
//! entry inside `\begin{small}…\end{small}` took the `\end` of an environment opened before it, once a group's frame
//! heard the `\begin`'s environment as opened there (`field_notes::note_environment`): 2512.19748.

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::set_fm_shadow_override;

use super::perfect_kernel_batch84b::creators;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// Convert `tex` without the census (production's default): the log and the XML.
fn run(tex: &str) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    &[],
    Some("ar5iv.sty"),
    move || {
      set_fm_shadow_override(None);
    },
    |_| (),
  );
  (log, xml)
}

const WRITTEN_WHOLE: &str = "The frontmatter emitter can't cut an author block (a name or an item of its reading joins \
                             ink across an alignment's cells): it is written whole, as Perl's whole-block creator";

/// `=2` writes a block whose reading crosses an alignment's cells as off does — one creator, the same element — with the
/// one counted Warning that says so, and no Error.
fn written_whole_as_off(tex: &str) {
  let (log, _) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(log.matches(WRITTEN_WHOLE).count(), 1, "{log}");
}

/// Names one per cell of a row (`Ann Able & Bob Baker & Cid Cole\\ Univ X & …`) read as one name joined across the cells
/// (`name:layout-joined`): the block _written whole. 1504.01048, 2607.22455, 2609.36783, 2507.07623, 2512.13460,
/// 2510.03886, 1902.06698, 2101.00440. The names' path: `joined_across_cells`.
#[test]
fn names_one_per_cell_are_written_whole() {
  written_whole_as_off(repro!("emit_names_one_per_cell_are_written_whole"));
}

/// A marked affiliation continued on from its column into the next row (`$^\dagger$Univ Y\\ Dept A & Dept B`): its entry
/// crosses a row end after a cell of a later column and a later column's cell: the block is written whole. 1304.3138,
/// 2511.14288, 2605.30319, 1711.10325, 1802.06894, 1704.06967. Distinct from the names' guard: an item's path,
/// `crosses_columns` over the entry's continued lines (`entry_span`).
#[test]
fn an_affiliation_run_into_the_next_column_is_written_whole() {
  written_whole_as_off(repro!(
    "emit_affiliation_run_into_the_next_column_is_written_whole"
  ));
}

/// An affiliation continued over two `{small}` lines, the second holding a group (`Universit{\`a}`): its line's run never
/// takes the `\end{small}` whose `\begin` is before it, so `=2` reads Ann Able's second affiliation whole and raises no
/// Error (2512.19748); off is unchanged, with no Error.
#[test]
fn a_line_in_an_environment_takes_no_foreign_end() {
  let tex = repro!("emit_line_in_an_environment_takes_no_foreign_end");
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&[
      "Ann Able",
      "affiliation=Univ X, City",
      "affiliation=Dept of Things, Università di A, City"
    ]),
    row(&[
      "Bob Baker",
      "affiliation=Univ X, City",
      "affiliation=Univ B, Town"
    ]),
  ]);
}
