//! Red/green guards for batch 98e: a block the marks cannot split lands once (OXIDIZED_DESIGN_DIVERGENCES #947). The D0
//! author lists (hep-ex0409040, hep-ex0612013: ~600 names `V.M.~Abazov,$^{33}$` over `\address{\centerline{$^{1}$…}
//! \centerline{$^{2}$…}…}`) gave every creator the whole 75-institute block, read as one unlabelled entry under REVTeX's
//! group rule; 2007.02542's BESIII group, one `{\it $^{1}$ …\\ … $^{l}$ …\\}` with four labels no name requests, gave
//! the holders of mark 1 the whole numbered list. Each guard converts its repros under
//! `tools/perfect_kernel/repros/class-bindings/` from the binary, eager and `--streaming`, asserts the same XML both
//! ways, no Error and no Warning, and every creator (name and contacts in order). One guard per path: a one-argument
//! line wrapper is a line (`relocate::wrapped_lines_as_lines`, at REVTeX's and AASTeX's call), the fail-safe of the
//! group rule (a block of several requested labels its lines unread lands once at the block), and the writer's cut of
//! a block's lines placing a line no name requests once at the block (`writer/plan.rs` `cut_block_lines`).
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batch89a::cli_conversion,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $name,
      ".tex"
    ))
  };
}

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error and no Warning and both give the
/// same XML, and returns the XML.
#[track_caller]
fn both_ways(tex: &str) -> String {
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (streamed_log, streamed) = cli_conversion(tex, true);
  assert_eq!(
    (error_count(&streamed_log), warning_count(&streamed_log)),
    (0, 0),
    "{streamed_log}"
  );
  assert_eq!(eager, streamed);
  eager
}

/// A one-argument line wrapper is a line: an `\address` block set with `\centerline` lines (hep-ex0409040,
/// hep-ex0612013), with a whole-line `\mbox`, two side by side each led by a mark (1412.5812) and a `\leftline`, and
/// AASTeX's `\affiliation` set so, each entry by its leading label, as the block written with `\\` (the control,
/// unchanged) is.
#[test]
fn line_wrapped_address_lines_go_by_their_marks() {
  let collaboration = "collaboration=(X Collaboration)";
  let lines = [
    vec![
      "A. One 1",
      collaboration,
      "affiliation=Alpha University, Beijing",
    ],
    vec![
      "B. Two 2",
      collaboration,
      "affiliation=Beta Institute, Rome",
    ],
    vec!["C. Three 3", collaboration, "affiliation=Gamma Lab, Paris"],
  ];
  for tex in [
    repro!("revtex_centerline_address_lines_go_by_their_marks"),
    repro!("revtex_address_lines_by_their_marks"),
  ] {
    assert_eq!(creators(&both_ways(tex)), lines);
  }
  assert_eq!(
    creators(&both_ways(repro!(
      "revtex_boxed_address_lines_go_by_their_marks"
    ))),
    [
      vec!["A. One 1", "affiliation=Alpha University, Beijing"],
      vec!["B. Two 2", "affiliation=Beta Institute, Rome"],
      vec!["C. Three 3", "affiliation=Gamma Lab, Paris"],
      vec!["D. Four 4", "affiliation=Delta College, Oslo"],
    ]
  );
  assert_eq!(
    creators(&both_ways(repro!(
      "aastex_centerline_affiliation_lines_go_by_their_marks"
    ))),
    [vec!["Ann Able 1", "affiliation=Uni One"], vec![
      "Bob Baker 2",
      "affiliation=Uni Two"
    ],]
  );
}

/// The group rule's fail-safe: a block holding two labels the names request, no mark leading it and its lines not read
/// as lines (a `\parbox`), lands once at the block — the names none — never a copy on each author.
#[test]
fn an_unsplit_labelled_address_block_lands_once_at_the_block() {
  let xml = both_ways(repro!("revtex_unsplit_labelled_address_block_lands_once"));
  assert_eq!(creators(&xml), [vec!["A. One 1"], vec!["B. Two 2"], vec![
    "affiliation=1 Alpha University, Beijing 2 Beta Institute, Rome"
  ],]);
}

/// The writer's cut of an author block's own lines: a line whose label no name requests (`$^{b}$`) goes once to the
/// block, the other lines to the names their marks name (2007.02542); with every label requested, or no such line, each
/// line goes by its marks (the controls).
#[test]
fn a_block_line_no_name_requests_goes_once_to_the_block() {
  let collaboration = "collaboration=(X Collaboration)";
  let a = vec![
    "A. One",
    collaboration,
    "affiliation=Alpha University, Beijing",
  ];
  let b = vec![
    "B. Two",
    collaboration,
    "affiliation=Beta Institute, Rome",
    "affiliation=Also at Delta College, Oslo",
  ];
  let c = vec!["C. Three", collaboration, "affiliation=Gamma Lab, Paris"];
  let xml = both_ways(repro!(
    "revtex_block_line_no_name_requests_goes_to_the_block"
  ));
  assert_eq!(creators(&xml), [a.clone(), b.clone(), c.clone(), vec![
    "affiliation=Also at Eps College, Bergen"
  ],]);
  let xml = both_ways(repro!("revtex_block_lines_all_answered"));
  assert_eq!(creators(&xml), [a.clone(), b.clone(), c.clone()]);
  let xml = both_ways(repro!("revtex_block_lines_every_label_requested"));
  let mut c_both = c;
  c_both.push("affiliation=Also at Eps College, Bergen");
  assert_eq!(creators(&xml), [a, b, c_both]);
}
