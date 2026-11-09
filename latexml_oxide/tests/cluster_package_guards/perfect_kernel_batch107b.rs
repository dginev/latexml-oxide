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

/// Converts `tex`, checks it has no Error and no Warning, and returns every creator's name and contacts
/// (`perfect_kernel_batch84b::creators`).
#[track_caller]
fn semantic_creators(tex: &str) -> Vec<Vec<String>> {
  let (log, xml) = convert_files_with(tex, &[], Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  perfect_kernel_batch84b::creators(&xml)
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
  for _ in expected {}
}
