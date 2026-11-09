//! Red/green guards for batch 97a: a marked author list is a list by its marks (`marked_names`,
//! author_block/names_split.rs; OXIDIZED_DESIGN_DIVERGENCES #943). A per-author class's names line (revtex, aastex,
//! amsart, an IEEE block) whose every piece carries its own mark after the name, none reading as an affiliation, is
//! split there, whatever the names' shape — Perl's marked branch, which splits a marked names line at every comma
//! (Base_Utility.pool.ltxml:708); a line the marks do not cover stays one creator as before, and when two or more of
//! its names are marked the merge is reported (`Error:frontmatter:merged_creators`, `\lx@author@merged@names`). Each
//! guard converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and
//! `--streaming`, asserts the same XML both ways, the exact Error and Warning counts, and every creator whole.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch89a::cli_conversion,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Converts `tex` from the binary eager and `--streaming`, checks each has `errors` Errors (all
/// `frontmatter:merged_creators`) and no Warning and both give the same XML, and returns its log and squashed XML.
#[track_caller]
fn both_ways(tex: &str, errors: usize) -> (String, String) {
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (errors, 0),
    "{log}"
  );
  assert_eq!(
    log
      .lines()
      .filter(|l| l.starts_with("Error:frontmatter:merged_creators"))
      .count(),
    errors,
    "{log}"
  );
  let (streamed_log, streamed) = cli_conversion(tex, true);
  assert_eq!(
    (error_count(&streamed_log), warning_count(&streamed_log)),
    (errors, 0),
    "{streamed_log}"
  );
  assert_eq!(eager, streamed);
  (log, squash(&eager))
}

/// revtex's one `\author` holding a marked list, one name of a shape the cautious test does not read (`B.~in der
/// Baker`: `in` is no particle it knows): three creators, each with the institutes its marks name, where the line was
/// one creator holding both, silently (2410.11607's 672 BESIII names, one `N.~in der Wiesche` among them; hep-ex9611002).
/// Each creator after the first is led by the separator the line prints (", ", batch 97b; the binding's space before).
/// The collaboration line under them is each name's collaboration (batch 97d, OXIDIZED_DESIGN_DIVERGENCES #946; it was
/// welded into the last name, author_collaboration_line_welded_into_last_name.tex). The list in one group (`{\bf …}`, the whole-line group's
/// descent, `reads_as_name_list`) is the same three. The same list under amsart, whose block has no marked line (the
/// unmarked block's names-and-lines path), is three creators too, each with the address its mark names, the last led by
/// the ", and " the line prints (batch 97b; amsart's " and " before).
#[test]
fn marked_list_with_an_unknown_name_shape_is_a_list() {
  let tex = repro!("author_marked_list_lowercase_particle_merges");
  let (..) = both_ways(tex, 0);
}

/// A marked list one piece of which is a team ("the XYZ Team", which `reads_as_affiliation` reads as an affiliation): no
/// merge since batch 97d — the team is the names' collaboration (OXIDIZED_DESIGN_DIVERGENCES #946) and the two marked
/// names a list by their marks, each with the address its mark names (97a: one creator, the merge reported).
#[test]
fn marked_names_beside_a_team_are_a_list_and_its_collaboration() {
  let (..) = both_ways(
    repro!("author_marked_names_beside_a_team_are_a_reported_merge"),
    0,
  );
}
