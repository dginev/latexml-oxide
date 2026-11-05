//! Red/green guards for batch 107b: a names list not parted (census138 row 3, MERGED; OXIDIZED_DESIGN_DIVERGENCES
//! #897). The splitters part the separators the census witnesses use, one rule per separator: a wrapper around the
//! block's lines is each line's (`wrapper_over_lines_as_lines`: a `\parbox`, a declaration environment or group, a
//! font command — 2311.03613, 1202.2048, 2306.04870, 2306.04988); a wrapper whose name ends in its marks ends where
//! they do (`space_parted_names`: 2608.29120); IEEE's dotless grades qualify the name before them (1906.01719); a
//! surname that is an institution's word after an initial is a marked name (1901.10843); authblk's "and" inside a
//! whole-list group (2506.23337); a sole `\input{file}` read as its content (2609.38746). And the detector
//! (`flag_merged_creators`) reads each printed line of a personname alone. Each guard converts its repro and asserts
//! the error and warning counts and every creator whole. Round 2 (review rev107b): a line of names horizontal space
//! alone parts is a names line, never an affiliation (`space_parted_rows_joined`; 1203.0321, 2507.19054); a space
//! before a tie is a tie in the observer's lexer and a legend line leads its line through its wrappers (1202.2048); a
//! creator named by a legend is an Error; `sole_input_file` matches its refusing commands whole.
use super::{
  convert_files_with,
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Every creator of `xml`, whitespace squashed.
fn creators(xml: &str) -> String {
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  xml[start..end].to_string()
}

/// Converts `tex` (beside `files`), checks its error count (`errors`) and that it warns nothing, and returns its
/// creators.
#[track_caller]
fn counted_creators(tex: &str, files: &[(&str, &str)], errors: usize) -> String {
  let (log, xml) = convert_files_with(tex, files, Some(PRELOAD));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (errors, 0),
    "{log}"
  );
  creators(&xml)
}

/// mnras: a `\parbox` around the names and their collaboration's `center` paragraph, a `\parbox` of marked affiliation
/// lines after a `\\` — each box's lines the block's (`wrapper_over_lines_as_lines`, the box path: its `\Large` kept on
/// each line, its layout dropped). Red (int138a): one creator, the names in an inline-block, silent. Witness
/// 2311.03613 (55 names; 2401.12049). Repro sectioning-frontmatter/author_parbox_names_above_collaboration_paragraph.
#[test]
fn a_parbox_over_names_and_their_collaboration_paragraph_is_its_lines() {
  assert_eq!(
    counted_creators(
      repro!("author_parbox_names_above_collaboration_paragraph"),
      &[],
      0,
    ),
    concat!(
      r#"<creator role="author"> "#,
      r#"<personname><text fontsize="144%" xml:id="id1">A. Able</text></personname> "#,
      r#"<contact role="collaboration"> "#,
      r#"<text fontsize="144%" xml:id="id2"> (XYZ Collaboration) </text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation"> University of Alpha</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation"> University of Beta</contact> </creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname><text fontsize="144%" xml:id="id3">B. C. Baker</text></personname> "#,
      r#"<contact role="collaboration"> "#,
      r#"<text fontsize="144%" xml:id="id4"> (XYZ Collaboration) </text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation"> University of Beta</contact> </creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname><text fontsize="144%" xml:id="id5">C. Cole</text></personname> "#,
      r#"<contact role="collaboration"> "#,
      r#"<text fontsize="144%" xml:id="id6"> (XYZ Collaboration) </text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation"> University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname><text fontsize="144%" xml:id="id7">D. Dee</text></personname> "#,
      r#"<contact role="collaboration"> "#,
      r#"<text fontsize="144%" xml:id="id8"> (XYZ Collaboration) </text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation"> University of Beta</contact> </creator>"#,
    )
  );
}

/// elsarticle, the BESIII template without its `center`: `\begin{small}` over the names, the collaboration line and an
/// `{\it …}` group of marked affiliation lines — the environment path, and a group inside a wrapped line unwrapped with
/// its style added. Red (int138a): one creator holding everything, silent. Witness 1202.2048 (327 names). Repro
/// sectioning-frontmatter/author_declaration_environment_over_names_and_affiliation_lines.
#[test]
fn a_declaration_environment_over_names_and_an_affiliation_group_is_its_lines() {
  assert_eq!(
    counted_creators(
      repro!("author_declaration_environment_over_names_and_affiliation_lines"),
      &[],
      0,
    ),
    concat!(
      r#"<creator role="author"> "#,
      r#"<personname><text fontsize="90%" xml:id="id1"> A. Able</text></personname> "#,
      r#"<contact role="collaboration"><text fontsize="90%" xml:id="id2"> (XYZ Collaboration)</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%" xml:id="id3"> University of Alpha, Alphaville</text></contact> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname><text fontsize="90%" xml:id="id4"> B. C. Baker</text></personname> "#,
      r#"<contact role="collaboration"><text fontsize="90%" xml:id="id5"> (XYZ Collaboration)</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%" xml:id="id6"> University of Beta, Betatown</text></contact> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname><text fontsize="90%" xml:id="id7"> C. Cole</text></personname> "#,
      r#"<contact role="collaboration"><text fontsize="90%" xml:id="id8"> (XYZ Collaboration)</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%" xml:id="id9"> University of Alpha, Alphaville</text></contact> "#,
      r#"</creator>"#,
    )
  );
}

/// aastex v5: `\textsc{…}` over marked names (two parted only by a mark and a newline) and a `(The … Collaboration)`
/// line — the font-command path, in a per-author class's cautious split. Red (int138a): one creator,
/// `merged_creators`. Witness 2306.04870 (20 names). Repro
/// sectioning-frontmatter/aastex_textsc_wrapper_over_names_and_collaboration_line.
#[test]
fn a_font_command_over_names_and_a_collaboration_line_is_its_lines() {
  assert_eq!(
    counted_creators(
      repro!("aastex_textsc_wrapper_over_names_and_collaboration_line"),
      &[],
      0,
    ),
    concat!(
      r#"<creator role="author"> "#,
      r#"<personname><text font="smallcaps" xml:id="id1">Ann Able<sup xml:id="id1.1"><text font="italic" xml:id="id1.1.1">1,2</text></sup></text></personname> "#,
      r#"<contact role="collaboration"><text font="smallcaps" xml:id="id2"> (The XYZ Collaboration)</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha, Alphaville</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta, Betatown</contact> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname><text font="smallcaps" xml:id="id3">Bob Baker<sup xml:id="id3.1"><text font="italic" xml:id="id3.1.1">3</text></sup></text></personname> "#,
      r#"<contact role="collaboration"><text font="smallcaps" xml:id="id4"> (The XYZ Collaboration)</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Gamma, Gammaton</contact> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname><text font="smallcaps" xml:id="id5">Cat Cole<sup xml:id="id5.1"><text font="italic" xml:id="id5.1.1">1</text></sup></text></personname> "#,
      r#"<contact role="collaboration"><text font="smallcaps" xml:id="id6"> (The XYZ Collaboration)</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha, Alphaville</contact> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname><text font="smallcaps" xml:id="id7">Dan Dee<sup xml:id="id7.1"><text font="italic" xml:id="id7.1.1">2</text></sup></text></personname> "#,
      r#"<contact role="collaboration"><text font="smallcaps" xml:id="id8"> (The XYZ Collaboration)</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta, Betatown</contact> "#,
      r#"</creator>"#,
    )
  );
}

/// A `\parbox` of `\quad`-parted names over two lines, then marked affiliations and a link line: the old reader parted
/// no names at the box's `\\` once a third line followed. Red (int138a): "Cat Cole <break/>Dan Dee" one creator,
/// silent. Witness 2306.04988. Repro sectioning-frontmatter/author_parbox_quad_names_over_a_break_then_link_line.
#[test]
fn a_parbox_of_quad_parted_names_over_two_lines_is_its_lines() {
  assert_eq!(
    counted_creators(
      repro!("author_parbox_quad_names_over_a_break_then_link_line"),
      &[],
      0,
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Dan Dee</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta </contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Eve Eden</personname> "#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id1">Corresponding author</note> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator role="author"> "#,
      r#"<contact name="URL: " role="url"><ref class="ltx_href" href="https://x.org/">https://x.org</ref></contact> "#,
      r#"</creator>"#,
    )
  );
}

/// `\textbf{Name\textsuperscript{n}} \⏎` names: a wrapper whose name ends in its marks ends where they do
/// (`space_parted_names`' wrapped-marks rule), the `\footnotemark`s each their name's. Red (int138a): "Dan Dee Eve Eden
/// Fay Fox†" one creator and Gus Gray lost, silent. Witness 2608.29120. Repro
/// sectioning-frontmatter/author_wrapped_marks_then_control_space_part_names.
#[test]
fn a_wrapper_ending_in_marks_then_a_control_space_parts_the_names() {
  assert_eq!(
    counted_creators(
      repro!("author_wrapped_marks_then_control_space_part_names"),
      &[],
      0,
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Gamma</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Dan Dee</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Eve Eden</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname>Fay Fox<note mark="†" role="footnote" xml:id="footnotex1"><tags> <tag>†</tag> "#,
      r#"<tag role="refnum">†</tag> <tag role="typerefnum">footnote †</tag> "#,
      r#"</tags>Corresponding authors.</note></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta</contact> </creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname>Gus Gray<note mark="†" role="footnote" xml:id="footnotex2"><tags> <tag>†</tag> "#,
      r#"<tag role="refnum">†</tag> <tag role="typerefnum">footnote †</tag> "#,
      r#"</tags>Corresponding authors.</note></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta</contact> </creator>"#,
    )
  );
}

/// An `\IEEEauthorblockN` list with `\textit{SMIEEE}`, `\textit{FIEEE}` pieces: IEEE's dotless grades qualify the name
/// before them (`NAME_GRADE_ABBREVIATIONS`), each its membership contact. Red (int138a): one creator,
/// `merged_creators`. Witness 1906.01719. Repro sectioning-frontmatter/ieee_dotless_membership_abbreviations_between_names.
#[test]
fn dotless_ieee_grades_between_names_are_their_memberships() {
  assert_eq!(
    counted_creators(
      repro!("ieee_dotless_membership_abbreviations_between_names"),
      &[],
      0,
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> "#,
      r#"<contact role="membership"><text font="italic" xml:id="id2">SMIEEE</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha, Alphaville</contact> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname>Bob Baker<sup xml:id="id3"><text font="italic" xml:id="id3.1">1,2</text></sup></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha, Alphaville</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta, Betatown</contact> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole<sup xml:id="id4">2</sup></personname> "#,
      r#"<contact role="membership"><text font="italic" xml:id="id5">FIEEE</text></contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta, Betatown</contact> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> "#,
      r#"<personname>Dan Dee<sup xml:id="id6"><text font="italic" xml:id="id6.1">1</text></sup></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha, Alphaville</contact> "#,
      r#"</creator>"#,
    )
  );
}

/// revtex4-1 one `\author{…}` list by its marks, one surname an institution's word after a middle initial (`Gary P.
/// Centers,$^{1}$`): a marked name (`surname_after_initial`), the printed separators kept. Red (int138a): one creator,
/// `merged_creators`. Witness 1901.10843. Repro sectioning-frontmatter/revtex_marked_list_surname_after_initial_is_a_name.
#[test]
fn an_institution_word_surname_after_an_initial_is_a_marked_name() {
  assert_eq!(
    counted_creators(
      repro!("revtex_marked_list_surname_after_initial_is_a_name"),
      &[],
      0,
    ),
    concat!(
      r#"<creator role="author"> "#,
      r#"<personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1,∗</text></sup></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=", " role="author"> "#,
      r#"<personname>Bob Baker<sup xml:id="id2"><text font="italic" xml:id="id2.1">1</text></sup></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=", " role="author"> "#,
      r#"<personname>Gary P. Centers<sup xml:id="id3"><text font="italic" xml:id="id3.1">1</text></sup></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=", " role="author"> "#,
      r#"<personname>Dan Dee<sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta</contact> </creator> "#,
      r#"<creator before=", and " role="author"> "#,
      r#"<personname>Eve Eden<sup xml:id="id5"><text font="italic" xml:id="id5.1">1,2</text></sup></personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta</contact> </creator>"#,
    )
  );
}

/// authblk: `{A\textsuperscript{a}\thanks{…} ~and B\textsuperscript{a}}` — a group around the whole list is
/// transparent to the "and" test (`has_top_level_and_word`). Red (int138a): one creator, `merged_creators`. Witness
/// 2506.23337. Repro sectioning-frontmatter/authblk_group_wrapped_tie_and_parts_names.
#[test]
fn authblk_and_inside_a_whole_list_group_parts_the_names() {
  assert_eq!(
    counted_creators(repro!("authblk_group_wrapped_tie_and_parts_names"), &[], 0,),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id1">Corresponding author. Email: ann@alpha.edu</note> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha, Alphaville</contact> "#,
      r#"</creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha, Alphaville</contact> "#,
      r#"</creator>"#,
    )
  );
}

/// `\author{\input{author_grid.tex}}`: the file's content in its place (`sole_input_file`). Red (int138a): one
/// creator holding every name and the affiliations' line, silent. Witness 2609.38746. Repro
/// sectioning-frontmatter/input_names_block/author_input_file_holding_the_names_block.
#[test]
fn a_sole_input_file_is_read_as_its_content() {
  assert_eq!(
    counted_creators(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/input_names_block/",
        "author_input_file_holding_the_names_block.tex"
      )),
      &[(
        "author_grid.tex",
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/input_names_block/",
          "author_grid.tex"
        )),
      )],
      0,
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id1"><text font="bold" xml:id="id1.1">Corresponding author: <text font="typewriter" xml:id="id1.1.1">ann@alpha.edu</text>.</text></note> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Cat Cole</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Dan Dee</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Eve Eden</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator>"#,
    )
  );
}

/// The detector reads each printed line of a personname alone (`flag_merged_creators`): two stacked names in a
/// `\shortstack` no splitter reads (two lines each a name), and a refused `\input` file's line of three names above a
/// line naming no one — each one `merged_creators` Error; a name over a mixed-case line holding an acronym (`IST
/// Austria`) none. Red (int138a): both merges silent. Witnesses 2306.04988, 2609.38746, 2311.03613; 2504.20039 (the
/// control). Repros sectioning-frontmatter/author_merged_lines_are_flagged,
/// input_names_block/author_input_file_refused_names_block_is_flagged, author_name_over_acronym_institution_is_not_flagged.
#[test]
fn merged_lines_in_one_personname_are_flagged() {
  assert_eq!(
    counted_creators(repro!("author_merged_lines_are_flagged"), &[], 1),
    concat!(
      r#"<creator role="author"> <personname><inline-block xml:id="id1"> <p xml:id="id1.1">Ann Able</p> "#,
      r#"<p xml:id="id1.2">Bob Baker</p> </inline-block></personname> </creator>"#,
    )
  );
  assert_eq!(
    counted_creators(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/input_names_block/",
        "author_input_file_refused_names_block_is_flagged.tex"
      )),
      &[(
        "author_grid_at.tex",
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/input_names_block/",
          "author_grid_at.tex"
        )),
      )],
      1,
    ),
    concat!(
      r#"<creator role="author"> "#,
      r#"<personname><text font="bold" xml:id="id1">Ann Able<sup xml:id="id1.1"><text font="italic" xml:id="id1.1.1">1</text></sup>, Bob Baker<sup xml:id="id1.2"><text font="italic" xml:id="id1.2.1">1</text></sup>, Cat Cole<sup xml:id="id1.3"><text font="italic" xml:id="id1.3.1">2</text></sup></text> "#,
      r#"<break/><sup xml:id="id2"><text font="italic" xml:id="id2.1">1</text></sup>University of Alpha <sup xml:id="id3"><text font="italic" xml:id="id3.1">2</text></sup>University of Beta</personname> "#,
      r#"</creator>"#,
    )
  );
  // (and a name over its institution in a mixed-case line holding an acronym is no two names: 2504.20039)
  assert_eq!(
    counted_creators(
      repro!("author_name_over_acronym_institution_is_not_flagged"),
      &[],
      0
    ),
    concat!(
      r#"<creator role="author"> <personname><inline-block xml:id="id1"> <p xml:id="id1.1">Ann Able</p> "#,
      r#"<p xml:id="id1.2">IST Austria</p> </inline-block></personname> </creator>"#,
    )
  );
}

/// Controls (R12): an article `\author{Ann Able \\ University of Alpha}` is one author with her affiliation, and so
/// is the same inside a `\parbox`, whose lines are the block's. Repros
/// sectioning-frontmatter/author_name_break_affiliation_control, author_wrapped_name_break_affiliation_control.
#[test]
fn a_name_and_the_line_under_it_stay_one_author_and_affiliation() {
  let control = concat!(
    r#"<creator role="author"> <personname>Ann Able</personname> "#,
    r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator>"#,
  );
  assert_eq!(
    counted_creators(repro!("author_name_break_affiliation_control"), &[], 0),
    control
  );
  assert_eq!(
    counted_creators(
      repro!("author_wrapped_name_break_affiliation_control"),
      &[],
      0
    ),
    control
  );
}

// Round 2 (review rev107b).

/// A line of names horizontal space alone parts is a names line, never an affiliation of the name above it
/// (`space_parted_rows_joined`, review rev107b B1): under the names line, more names of its row — at the top level, in
/// a `\parbox` (introduced by 107b r1: int138a parted the four), two rows over the institution they share —; under a
/// line under the names, the next group's names, each row its own affiliation. Red (107b r1): Bob Baker held the
/// affiliation "Cat Cole" (r04: Cat Cole "Dan Dee"), silent. Repros sectioning-frontmatter/
/// author_quad_names_lines_unmarked_are_one_row, author_parbox_quad_names_lines_unmarked_are_one_row,
/// author_parbox_quad_names_rows_share_the_line_under, author_quad_names_line_after_an_affiliation_opens_the_next_group.
#[test]
fn a_space_parted_names_line_is_names_never_an_affiliation() {
  let four = concat!(
    r#"<creator role="author"> <personname>Ann Able</personname> </creator> "#,
    r#"<creator before=" " role="author"> <personname>Bob Baker</personname> </creator> "#,
    r#"<creator before=" " role="author"> <personname>Cat Cole</personname> </creator> "#,
    r#"<creator before=" " role="author"> <personname>Dan Dee</personname> </creator>"#,
  );
  assert_eq!(
    counted_creators(
      repro!("author_quad_names_lines_unmarked_are_one_row"),
      &[],
      0
    ),
    four
  );
  assert_eq!(
    counted_creators(
      repro!("author_parbox_quad_names_lines_unmarked_are_one_row"),
      &[],
      0
    ),
    four
  );
  let at = |name: &str, before: bool, university: &str| {
    format!(
      r#"<creator {}role="author"> <personname>{name}</personname> <contact name="Affiliation: " role="affiliation">{university}</contact> </creator>"#,
      if before { r#"before=" " "# } else { "" }
    )
  };
  let alpha = "University of Alpha";
  assert_eq!(
    counted_creators(
      repro!("author_parbox_quad_names_rows_share_the_line_under"),
      &[],
      0
    ),
    [
      at("Ann Able", false, alpha),
      at("Bob Baker", true, alpha),
      at("Cat Cole", true, alpha),
      at("Dan Dee", true, alpha),
      at("Eve Eden", true, alpha),
    ]
    .join(" ")
  );
  assert_eq!(
    counted_creators(
      repro!("author_quad_names_line_after_an_affiliation_opens_the_next_group"),
      &[],
      0
    ),
    [
      at("Ann Able", false, alpha),
      at("Bob Baker", true, alpha),
      at("Cat Cole", true, "University of Beta"),
      at("Dan Dee", true, "University of Beta"),
    ]
    .join(" ")
  );
}

/// A font command over one line of `\quad`-parted names parts them (`space_parted_name_line` through the whole-line
/// wrapper, review rev107b S2): under a marked row (the marked reader: more names, not the last name's continuation) and
/// under a row marked by footnote symbols alone (the unmarked reader's rows). Red (107b r1): Cat Cole held the
/// wrapper's names, one merged_creators Error. Witness 2507.19054. Repros sectioning-frontmatter/
/// author_bold_quad_names_line_under_a_marked_row, author_bold_quad_names_line_under_a_symbol_marked_row.
#[test]
fn a_font_command_over_quad_parted_names_parts_them() {
  let at = |name: &str, before: bool| {
    format!(
      r#"<creator {}role="author"> <personname>{name}</personname> <contact name="Affiliation: " role="affiliation">Stanford University</contact> </creator>"#,
      if before { r#"before=" " "# } else { "" }
    )
  };
  assert_eq!(
    counted_creators(
      repro!("author_bold_quad_names_line_under_a_marked_row"),
      &[],
      0
    ),
    [
      at(r#"Ann Able<sup xml:id="id1">1</sup>"#, false),
      at("Cat Cole", true),
      at("Bob Baker", true),
      at("Dan Dee", true),
    ]
    .join(" ")
  );
  assert_eq!(
    counted_creators(
      repro!("author_bold_quad_names_line_under_a_symbol_marked_row"),
      &[],
      0
    ),
    [
      at(r#"Ann Able<sup xml:id="id1">⋆</sup>"#, false),
      at(r#"Bob Baker<sup xml:id="id2">⋆,†</sup>"#, true),
      at("Cat Cole", true),
      at("Dan Dee", true),
      at("Eve Eden", true),
    ]
    .join(" ")
  );
}

/// The BESIII-shaped block's creators, each its name and contacts in order ([`semantic_creators`]). The sub-lettered
/// names' visible marks ("B. Baker 10a") are a known residual pinned as is (FIDELITY_CHARTER §4 rank 93: a name's label
/// is the name alone; review rev107b2 F3), not settled design.
fn besiii_creators() -> Vec<Vec<&'static str>> {
  let collaboration = "collaboration=(X Collaboration)";
  vec![
    vec!["A. Able", collaboration, "affiliation=Univ N1, City"],
    vec![
      "B. Baker 10a",
      collaboration,
      "affiliation=Univ N10, City",
      "affiliation=also at Inst Alpha, Moscow",
    ],
    vec!["C. Cole", collaboration, "affiliation=Univ N2, City"],
    vec![
      "D. Dee 12b",
      collaboration,
      "affiliation=Univ N12, City",
      "affiliation=on leave from Inst Beta, Kiev",
    ],
    vec!["Lei Li", collaboration, "affiliation=Univ N1, City"],
    vec!["N. B. Li", collaboration, "affiliation=Univ N11, City"],
    vec!["Q. J. Li", collaboration, "affiliation=Univ N1, City"],
    // (the institutes no name requests, once on the block's creator)
    vec![
      "affiliation=Univ N3, City",
      "affiliation=Univ N4, City",
      "affiliation=Univ N5, City",
      "affiliation=Univ N6, City",
      "affiliation=Univ N7, City",
      "affiliation=Univ N8, City",
      "affiliation=Univ N9, City",
    ],
  ]
}

/// Converts `tex`, checks it has no Error and no Warning, and returns every creator's name and contacts
/// (`perfect_kernel_batch84b::creators`).
#[track_caller]
fn semantic_creators(tex: &str) -> Vec<Vec<String>> {
  let (log, xml) = convert_files_with(tex, &[], Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  perfect_kernel_batch84b::creators(&xml)
}

/// A space before a tie is a tie (the lexer, review rev107b B2): the BESIII block with `N.~B. ~Li$^{11}$` reads as the
/// control without the space — the reading's names as printed, its sub-lettered marks kept (`$^{10a}$` the
/// institute 10 and the `a` legend; the glued mark shown on the name, as a mark the writer parts is, 2007.02542's
/// `D.~Four$^{23A}$`) — and its `$^{a}$` legend after `\vspace` inside the line's two wrappers leads its line
/// (`marker_leads`), never an author. Red (107b r1): a creator named "also at Inst Alpha, Moscow", B. Baker and D. Dee
/// no institute and no legend, silent. Witness 1202.2048. Repros sectioning-frontmatter/
/// author_besiii_space_before_tie_keeps_sub_lettered_marks, author_besiii_sub_lettered_marks_tie_control.
#[test]
fn a_space_before_a_tie_keeps_the_blocks_sub_lettered_marks() {
  assert_eq!(
    semantic_creators(repro!("author_besiii_sub_lettered_marks_tie_control")),
    besiii_creators()
  );
  assert_eq!(
    semantic_creators(repro!(
      "author_besiii_space_before_tie_keeps_sub_lettered_marks"
    )),
    besiii_creators()
  );
}

/// 1202.2048's BESIII block as written (review rev107b B2): 340 creators, every one named, no legend a creator, and each
/// sub-lettered name its number's institute and its letter's legend (`E.~Boger$^{18a}$`, `I.~Denysenko$^{18b}$`,
/// `S.~P.~Pacetti$^{17c}$`, `A.~Sarantsev$^{18d}$`, `C.~P.~Shen$^{36e}$`), each legend on the names requesting it alone,
/// and the spaced-tie `N.~B. ~Li$^{22}$` one name. Red (107b r1): 342 creators, the `a` legend one and a nameless one
/// holding the others, the sub-lettered names no institute 18. Repro
/// sectioning-frontmatter/author_besiii_block_sub_lettered_marks_1202_2048.
#[test]
fn the_1202_2048_besiii_block_gives_every_name_its_lines() {
  let creators = semantic_creators(repro!("author_besiii_block_sub_lettered_marks_1202_2048"));
  assert_eq!(creators.len(), 340);
  assert!(
    creators
      .iter()
      .all(|c| c.first().is_some_and(|name| !name.contains('='))),
    "every creator named: {creators:?}"
  );
  let collaboration = "collaboration=(BESIII Collaboration)";
  let jinr = "affiliation=Joint Institute for Nuclear Research, 141980 Dubna, Russia";
  let a = "affiliation=also at the Moscow Institute of Physics and Technology, Moscow, Russia";
  let expected: [&[&str]; 9] = [
    &["E. Boger 18a", collaboration, jinr, a],
    &["G. Chelkov 18a", collaboration, jinr, a],
    &["A. Zhemchugov 18a", collaboration, jinr, a],
    &[
      "I. Denysenko 18b",
      collaboration,
      jinr,
      "affiliation=on leave from the Bogolyubov Institute for Theoretical Physics, Kiev, Ukraine",
    ],
    &[
      "S. P. Pacetti 17c",
      collaboration,
      "affiliation=INFN Laboratori Nazionali di Frascati , Frascati, Italy",
      "affiliation=Currently at University of Perugia and INFN, Perugia, Italy",
    ],
    &[
      "A. Sarantsev 18d",
      collaboration,
      jinr,
      "affiliation=also at the PNPI, Gatchina, Russia",
    ],
    &[
      "C. P. Shen 36e",
      collaboration,
      "affiliation=University of Hawaii, Honolulu, Hawaii 96822, USA",
      "affiliation=now at Nagoya University, Nagoya, Japan",
    ],
    &[
      "N. B. Li",
      collaboration,
      "affiliation=Nanjing Normal University, Nanjing 210046, P. R. China",
    ],
    &[
      "W. L. Yuan",
      collaboration,
      "affiliation=Nanjing Normal University, Nanjing 210046, P. R. China",
    ],
  ];
  for creator in expected {
    let found: Vec<&Vec<String>> = creators.iter().filter(|c| c[0] == creator[0]).collect();
    assert_eq!(found, [&creator
      .iter()
      .map(|s| s.to_string())
      .collect::<Vec<_>>()]);
  }
  // (each legend on the names its letter's marks request, and on no other)
  for (legend, holders) in [
    ("Moscow Institute of Physics", 3),
    ("Bogolyubov Institute", 1),
    ("University of Perugia", 1),
    ("PNPI, Gatchina", 1),
    ("now at Nagoya University", 1),
  ] {
    assert_eq!(
      creators
        .iter()
        .filter(|c| c.iter().any(|item| item.contains(legend)))
        .count(),
      holders,
      "{legend}"
    );
  }
}

/// The detector's readings of round 2 (review rev107b B2, N2): a creator named by a legend's text (`also at …`) is a
/// `creator_not_a_name` Error, never silent; a name over a line under it, each line reading as a name, is counted by
/// its lines and worded so. Repros sectioning-frontmatter/author_legend_line_read_as_an_author_is_flagged,
/// author_name_over_unmarked_line_counted_by_its_lines. The creators pinned here are KNOWN WRONG and Error-flagged: a
/// later fix re-pins them, never "repairs" this guard to pass (review rev107b2 F5).
#[test]
fn a_creator_named_by_a_legend_is_flagged() {
  let (log, xml) = convert_files_with(
    repro!("author_legend_line_read_as_an_author_is_flagged"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.contains(concat!(
      "Error:frontmatter:creator_not_a_name An author's name reads as no name: \"also at the Moscow Institute of ",
      "Physics\"; a line under the names (an affiliation or a legend) was read as an author"
    )),
    "{log}"
  );
  assert_eq!(perfect_kernel_batch84b::creators(&xml), [
    vec!["Ann Able"],
    vec!["also at the Moscow Institute of Physics"]
  ]);
  let (log, xml) = convert_files_with(
    repro!("author_name_over_unmarked_line_counted_by_its_lines"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.contains(concat!(
      "Error:frontmatter:merged_creators One author holds 2 lines that each read as a name: \"Bob Baker / Carnegie ",
      "Mellon\"; the names a line break parts, or a name and the line under it (its affiliation), were not told apart"
    )),
    "{log}"
  );
  assert_eq!(perfect_kernel_batch84b::creators(&xml), [
    vec!["Ann Able 1"],
    vec!["Bob Baker Carnegie Mellon 2"]
  ]);
}

/// A sole `\input` file opening with `\inputencoding` is read as its content (`sole_input_file` matches its refusing
/// commands whole, review rev107b N3). Red (107b r1): one creator, 1 merged_creators Error. Repro
/// sectioning-frontmatter/input_names_block/author_input_file_with_inputencoding_is_read.
#[test]
fn a_sole_input_file_opening_with_inputencoding_is_read() {
  assert_eq!(
    counted_creators(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/input_names_block/",
        "author_input_file_with_inputencoding_is_read.tex"
      )),
      &[(
        "author_grid_enc.tex",
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/input_names_block/",
          "author_grid_enc.tex"
        )),
      )],
      0,
    ),
    concat!(
      r#"<creator role="author"> <personname>Ann Able</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Alpha</contact> </creator> "#,
      r#"<creator before=" " role="author"> <personname>Bob Baker</personname> "#,
      r#"<contact name="Affiliation: " role="affiliation">University of Beta</contact> </creator>"#,
    )
  );
}
