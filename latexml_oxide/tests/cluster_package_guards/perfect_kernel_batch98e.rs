//! Red/green guards for batch 98e: a block the marks cannot split lands once (OXIDIZED_DESIGN_DIVERGENCES #947). The D0
//! author lists (hep-ex0409040, hep-ex0612013: ~600 names `V.M.~Abazov,$^{33}$` over `\address{\centerline{$^{1}$…}
//! \centerline{$^{2}$…}…}`) gave every creator the whole 75-institute block, read as one unlabelled entry under REVTeX's
//! group rule; 2007.02542's BESIII group, one `{\it $^{1}$ …\\ … $^{l}$ …\\}` with four labels no name requests, gave
//! the holders of mark 1 the whole numbered list. Each guard converts its repros under
//! `tools/perfect_kernel/repros/class-bindings/` from the binary, eager and `--streaming`, asserts the same XML both
//! ways, no Error and no Warning, and every creator (name and contacts in order). One guard per path: a one-argument
//! line wrapper is a line (`relocate::wrapped_lines_as_lines`, at REVTeX's and AASTeX's call), the fail-safe of the
//! group rule (a block of two or more lines led by requested labels its reader cannot cut lands once at the block),
//! the writer's cut of a block's lines placing a line no name requests once at the block with its printed label
//! (`writer/plan.rs` `cut_block_lines`); round 2 (review rev98e): a mark on a spacing nucleus leads its line
//! (`relocate::spaced_marks_as_marks`, 2112.13115, 0807.4407), and a sub-lettered mark goes by its number
//! (`resolver/labels.rs` `parted_label`, 2007.02542, 1507.02068).
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
  // (the same under `\affiliation` without `groupedaddress`, in a `\parbox` — review rev98e S2's fs3 — and in a
  // `tabular`, its opener past, 0911.3234)
  for tex in [
    repro!("revtex_parbox_affiliation_lines_land_once"),
    repro!("revtex_tabular_affiliation_lines_land_once"),
  ] {
    assert_eq!(creators(&both_ways(tex)), [
      vec!["Ann Able 1"],
      vec!["Bob Baker 2"],
      vec!["affiliation=1 Univ A, Town 2 Univ B, City"],
    ]);
  }
}

/// The fail-safe fires only for a block of two or more lines led by labels the names request (review rev98e S2): lines
/// led by a mark in `\( … \)` are read by the shared reader of a line's leading label (`relocate::spaced_mark_at`), each
/// author its line, as `$^{1}$` lines are (review N1's shape; the coordinator's 2026-11-01 directive: one reader, not a
/// fallback to the observer); marks inside one line's text (`Joint Institute, \textsuperscript{1}Univ A and
/// \textsuperscript{2}Univ B, City`) are no such block, but the group's one entry, a copy on each author (97b, 98a).
#[test]
fn the_fail_safe_fires_only_for_labelled_lines() {
  let xml = both_ways(repro!("revtex_unread_mark_lines_go_by_their_marks"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "affiliation=Univ A, Town"],
    vec!["Bob Baker 2", "affiliation=Univ B, City"],
  ]);
  let xml = both_ways(repro!("revtex_marks_inside_one_line_are_the_groups_entry"));
  let line = "affiliation=Joint Institute, 1 Univ A and 2 Univ B, City";
  assert_eq!(creators(&xml), [vec!["Ann Able 1", line], vec![
    "Bob Baker 2",
    line
  ]]);
}

/// A mark set on a spacing nucleus leads its line as `$^{1}$` does (`relocate::spaced_marks_as_marks`, review rev98e
/// S2): `${\,\!}^1$…\\ and ${\,\!}^2$…` cut by its marks, each author its departments (2112.13115), and a box holding
/// only such a mark, `\mbox{${\ }^{1}$}` (0807.4407), and spacing unbraced, `$~^{1}$` (1211.5032).
#[test]
fn a_mark_on_a_spacing_nucleus_leads_its_line() {
  let xml = both_ways(repro!("revtex_nucleus_marks_lead_their_lines"));
  assert_eq!(creators(&xml), [
    // (each e-mail after the departments: REVTeX prints `\email` in the front matter's footnotes, its print zone the
    // footer's — 99a r3, OXIDIZED_DESIGN_DIVERGENCES #954)
    vec![
      "Ann Able 1 , 2",
      "affiliation=Department of Mathematics",
      "affiliation=and Department of Physics, The Univ, City, USA",
      "email=able@x.edu",
    ],
    vec![
      "Bob Baker 1",
      "affiliation=Department of Mathematics",
      "email=baker@x.edu",
    ],
  ]);
  let xml = both_ways(repro!("revtex_boxed_nucleus_marks_lead_their_lines"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "affiliation=Univ A, Town"],
    vec!["Bob Baker 2", "affiliation=Univ B, City"],
  ]);
  // (the script braced whole, `${^1}$`: 2208.09667's lines after its first)
  let xml = both_ways(repro!("revtex_braced_script_marks_lead_their_lines"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "affiliation=Univ A, Town"],
    vec!["Bob Baker 2", "affiliation=Univ B, City"],
  ]);
  // (spacing unbraced, `$~^{1}$`: 1211.5032)
  let xml = both_ways(repro!("revtex_tie_nucleus_marks_lead_their_lines"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "affiliation=Univ A, Town"],
    vec![
      "Bob Baker 1,2",
      "affiliation=Univ A, Town",
      "affiliation=Univ B, City",
    ],
  ]);
}

/// A sub-lettered mark goes by its number (review rev98e B1): `23A` under multi-digit numbering is `23` and `A`, never
/// `2`, `3` and `A`, and the `$^{23}$` line is never parted into `2` and `3` — D. Four the 23 line alone (its `(A)` a
/// sub-entry, R6), B. Two and C. Three not (2007.02542, 1507.02068); with a line no name requests, it goes once to the
/// block with its printed `b` (S1), and without one the same reading.
#[test]
fn a_sub_lettered_mark_goes_by_its_number() {
  let collaboration = "collaboration=(X Collaboration)";
  let mut expected = vec![
    vec![
      "A. One",
      collaboration,
      "affiliation=Alpha University, Beijing",
    ],
    vec![
      "B. Two",
      collaboration,
      "affiliation=Beta Institute, Rome",
      "affiliation=Also at Delta College, Oslo",
    ],
    vec!["C. Three", collaboration, "affiliation=Gamma Lab, Paris"],
    vec![
      "D. Four 23A",
      collaboration,
      "affiliation=(A)Zeta Lab, Turin; (B)Eta Lab, Milan",
    ],
  ];
  let xml = both_ways(repro!("revtex_sub_lettered_mark_every_line_answered"));
  assert_eq!(creators(&xml), expected);
  expected.push(vec!["affiliation=b Also at Eps College, Bergen"]);
  let xml = both_ways(repro!("revtex_sub_lettered_mark_goes_by_its_number"));
  assert_eq!(creators(&xml), expected);
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
  // (the line keeps its printed `b`: label text is markup, never dropped — review rev98e S1)
  assert_eq!(creators(&xml), [a.clone(), b.clone(), c.clone(), vec![
    "affiliation=b Also at Eps College, Bergen"
  ],]);
  let xml = both_ways(repro!("revtex_block_lines_all_answered"));
  assert_eq!(creators(&xml), [a.clone(), b.clone(), c.clone()]);
  let xml = both_ways(repro!("revtex_block_lines_every_label_requested"));
  let mut c_both = c;
  c_both.push("affiliation=Also at Eps College, Bergen");
  assert_eq!(creators(&xml), [a, b, c_both]);
}
