//! Guards for NE8 (`agent_reports/2026-11-09_brief_NE8.md`, review revX4 2): a name's run holding an `\if…` whose `\fi`
//! is outside it is refused as an `\else`/`\fi` cut from its `\if` is (NE5, 1409.0597) — digested apart, it stays open to
//! the end of the document. (The IJCAI address section, revX4 1, is `perfect_kernel_batchne6`'s re-expected guard.)

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

use super::perfect_kernel_batch84b::creators;

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// Convert `tex` under `emit`: the log and the XML.
fn run(tex: &str, emit: u8) -> (String, String) {
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
  (log, xml)
}

/// `\newif\iffull\fulltrue` and `\iffull Bob Baker\fnm{2}\fi`: `=2` wrote the name from the run `\iffull Bob Baker`, its
/// `\fi` outside it, and the conditional stayed open ("Attempt to end document with open … conditionals", off none). An
/// opener cut from its `\fi` is refused too (`content::conditional_cut`): the name is its text, a counted Warning, the
/// creators off's.
#[test]
fn a_name_run_holding_an_open_conditional_is_its_text() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ne8_cond_opener_cut.tex"
  );
  let expected = [
    row(&["Ann Able", "affiliation=Univ A"]),
    row(&["Bob Baker", "affiliation=Univ B"]),
    row(&["Carl Cole", "affiliation=Univ A"]),
  ];
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(r#"The frontmatter emitter writes "Bob Baker" as its plain text"#),
    "{log}"
  );
  assert!(
    !log.contains("open groups, environments or conditionals"),
    "{log}"
  );
  assert_eq!(creators(&xml), expected);
  let (log, xml) = run(tex, 0);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), expected);
}
