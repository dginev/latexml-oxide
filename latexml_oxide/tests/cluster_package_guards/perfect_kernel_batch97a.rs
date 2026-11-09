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
  perfect_kernel_batch77d::creators,
  perfect_kernel_batch89a::cli_conversion,
  perfect_kernel_batcht1::red_at_two,
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
  let (_, xml) = both_ways(tex, 0);
  if red_at_two(r#"writes before=" " where the list prints ", " between the names"#) {
    return;
  }
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>A. Able</personname> <contact role="collaboration">(XYZ Collaboration)</contact> <contact name="Affiliation: " role="affiliation"> Institute of One</contact> </creator>"#,
    r#"<creator before=", " role="author"> <personname>B. in der Baker</personname> <contact role="collaboration">(XYZ Collaboration)</contact> <contact name="Affiliation: " role="affiliation"> University of Two</contact> </creator>"#,
    r#"<creator before=", " role="author"> <personname>C. Cole</personname> <contact role="collaboration">(XYZ Collaboration)</contact> <contact name="Affiliation: " role="affiliation"> Institute of One</contact> </creator>"#,
  ]);
  let grouped = tex.replace(
    r"\author{A.~Able$^{1}$, B.~in der Baker$^{2}$, C.~Cole$^{1}$\\",
    r"\author{{\bf A.~Able$^{1}$, B.~in der Baker$^{2}$, C.~Cole$^{1}$}\\",
  );
  assert_ne!(grouped, tex);
  let (_, xml) = both_ways(&grouped, 0);
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>A. Able</personname> <contact role="collaboration">(XYZ Collaboration)</contact> <contact name="Affiliation: " role="affiliation"> Institute of One</contact> </creator>"#,
    r#"<creator before=", " role="author"> <personname>B. in der Baker</personname> <contact role="collaboration">(XYZ Collaboration)</contact> <contact name="Affiliation: " role="affiliation"> University of Two</contact> </creator>"#,
    r#"<creator before=", " role="author"> <personname>C. Cole</personname> <contact role="collaboration">(XYZ Collaboration)</contact> <contact name="Affiliation: " role="affiliation"> Institute of One</contact> </creator>"#,
  ]);
  let amsart = "\\documentclass{amsart}\n\\begin{document}\n\\title{A Title}\n\\author{A.~Able$^{1}$, B.~in der \
                Baker$^{2}$, and C.~Cole$^{1}$}\n\\address{$^{1}$ Institute of One; $^{2}$ University of \
                Two}\n\\maketitle\nBody text.\n\\end{document}\n";
  let (_, xml) = both_ways(amsart, 0);
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>A. Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup></personname> <contact name="Address: " role="address"><sup xml:id="id2"><text font="italic" xml:id="id2.1">1</text></sup> Institute of One</contact> </creator>"#,
    r#"<creator before=", " role="author"> <personname>B. in der Baker<sup xml:id="id3"><text font="italic" xml:id="id3.1">2</text></sup></personname> <contact name="Address: " role="address"><sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup> University of Two</contact> </creator>"#,
    r#"<creator before=", and " role="author"> <personname>C. Cole<sup xml:id="id5"><text font="italic" xml:id="id5.1">1</text></sup></personname> <contact name="Address: " role="address"><sup xml:id="id6"><text font="italic" xml:id="id6.1">1</text></sup> Institute of One</contact> </creator>"#,
  ]);
}

/// A marked list one of whose names has no mark is no list by its marks: the cautious test keeps it one creator, as
/// before, and the two marked names it holds are reported, once — the marked names counted, a lower bound (the
/// marked-list reading does not widen to a list a mark does not cover); in one group (`{\bf …}`) the names are counted
/// inside it, the same report. Without the collaboration line the observer's writer re-parts the names into three
/// creators, which carry no merge count: no Error.
#[test]
fn marked_list_with_an_unmarked_name_is_a_reported_merge() {
  let tex = repro!("author_marked_list_with_an_unmarked_name_is_a_reported_merge");
  if red_at_two(r#"unverified: no merged_creators Error where the off reading reports the merge"#) {
    return;
  }
  let (log, xml) = both_ways(tex, 1);
  assert!(log.contains("One author holds 2 names"), "{log}");
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>A. Able, B. in der Baker, C. Cole</personname> <contact role="collaboration">(XYZ Collaboration)</contact> <contact name="Affiliation: " role="affiliation"> Institute of One</contact> </creator>"#,
  ]);
  let grouped = tex.replace(
    r"\author{A.~Able$^{1}$, B.~in der Baker, C.~Cole$^{1}$\\",
    r"\author{{\bf A.~Able$^{1}$, B.~in der Baker, C.~Cole$^{1}$}\\",
  );
  assert_ne!(grouped, tex);
  let (log, xml) = both_ways(&grouped, 1);
  assert!(log.contains("One author holds 2 names"), "{log}");
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>A. Able, B. in der Baker, C. Cole</personname> <contact role="collaboration">(XYZ Collaboration)</contact> <contact name="Affiliation: " role="affiliation"> Institute of One</contact> </creator>"#,
  ]);
  let written = tex.replace("(XYZ Collaboration)\\\\\n", "");
  assert_ne!(written, tex);
  let (_, xml) = both_ways(&written, 0);
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>A. Able</personname> <contact name="Affiliation: " role="affiliation"> Institute of One</contact> </creator>"#,
    r#"<creator role="author"> <personname>B. in der Baker</personname> </creator>"#,
    r#"<creator role="author"> <personname>C. Cole</personname> <contact name="Affiliation: " role="affiliation"> Institute of One</contact> </creator>"#,
  ]);
}

/// A marked list one piece of which is a team ("the XYZ Team", which `reads_as_affiliation` reads as an affiliation): no
/// merge since batch 97d — the team is the names' collaboration (OXIDIZED_DESIGN_DIVERGENCES #946) and the two marked
/// names a list by their marks, each with the address its mark names (97a: one creator, the merge reported).
#[test]
fn marked_names_beside_a_team_are_a_list_and_its_collaboration() {
  let (_, xml) = both_ways(
    repro!("author_marked_names_beside_a_team_are_a_reported_merge"),
    0,
  );
  if red_at_two(r#"writes before=" and " where the list prints ", " between the names"#) {
    return;
  }
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>A. Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup></personname> <contact role="collaboration">the XYZ Team</contact> <contact name="Address: " role="address"><sup xml:id="id2"><text font="italic" xml:id="id2.1">1</text></sup> Institute of One</contact> </creator>"#,
    r#"<creator before=", " role="author"> <personname>B. in der Baker<sup xml:id="id3"><text font="italic" xml:id="id3.1">2</text></sup></personname> <contact role="collaboration">the XYZ Team</contact> <contact name="Address: " role="address"><sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup> University of Two</contact> </creator>"#,
  ]);
}
