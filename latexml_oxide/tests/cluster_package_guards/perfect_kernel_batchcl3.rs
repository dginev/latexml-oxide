//! Guards for CL3 (`agent_reports/2026-11-08_brief_CL3.md`; w56's report, SYNC leftovers (33)): under
//! `=2` a block the emitter writes whole (`observer/emitter/emit.rs` `whole_block`) has its tabular juggles
//! and columns written as the line breaks they print (`fallback::tabular_juggles_in_a_whole_block`). vldb.cls's
//! `\alignauthor`, inside the whole creator's `\lx@personname`, closed a tabular the block never opened: 4 Errors and all
//! 8 names lost (1511.05911, its rehearsal over the read budget).

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz3::in_a_directory};

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// A vldb.cls-shaped block no rehearsal reads (a guard's small budget): off reads it as it prints, two creators, the
/// budget's Warning only; `=2` writes it whole, one creator holding every word, its column `\alignauthor` gone and its
/// line breaks kept, with no Error (the base's `=2`: four mode Errors, the name empty). One guard: the whole-block writer
/// is one code path, the over-budget block its witness's way there.
#[test]
fn a_whole_blocks_tabular_juggle_is_a_line_break() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/emit_whole_block_tabular_juggle.tex"
  );
  let class =
    include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/vldbjuggle.cls");
  let whole = [row(&[
    "Ann Able 1 Bob Baker 2 1 Univ Alpha 2 Univ Beta",
    "email=ann@alpha.edu",
    "email=bob@beta.edu",
  ])];
  let warnings = 2;
  let expected = &whole[..];
  let (log, xml) = in_a_directory(tex, &[("vldbjuggle.cls", class)], &[(
    "LATEXML_REHEARSAL_READ_BUDGET",
    "20000",
  )]);
  assert_eq!(
    (error_count(&log), log.matches("\nFatal:").count()),
    (0, 0),
    "{log}"
  );
  assert_eq!(warning_count(&log), warnings, "{log}");
  assert_eq!(
    log.matches("Warning:frontmatter:rehearsal_budget").count(),
    1,
    "{log}"
  );
  assert_eq!(creators(&xml), expected, "{xml}");
}
