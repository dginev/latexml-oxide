//! Guards for NE4 (`agent_reports/2026-11-09_brief_NE4.md`): the flip's `=2`-only Errors. A block's own keyvals are written
//! on its creators as the call's tokens, never as their text retokenized, where a command glues to the letters after it
//! (lni's `\author[Christian Burkert \and Jonathan Balack …]` read back `\andJonathan`: 2205.13776, 2406.04210, 2302.12131,
//! 2307.15427, 2309.09081, 2310.00933, 2407.04435, 2210.04484).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};

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

/// Convert `tex`: the log and the XML.
fn run(tex: &str) -> (String, String) {
  let (log, xml, ()) =
    convert_files_with_setup_then(tex, &[], Some("ar5iv.sty"), move || {}, |_| ());
  (log, xml)
}

/// authblk's `\author[\dag a]{Ann Able}`: the block's `annotations={\dag a}` hold a command, so `=2` carries them on its
/// creators as off digests them — written from the ledger's text, retokenized, `\dag a` was `\daga`, an Error per author.
/// Now the call's own tokens are written, the creators off's.
#[test]
fn a_command_in_an_authors_annotations_stays_one_token() {
  let (log, xml) = run(repro!("ne4_authblk_command_glued_in_annotations"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able"]),
    row(&["Bob Baker"]),
    row(&["affiliation=University A", "affiliation=University B"])
  ]);
}
