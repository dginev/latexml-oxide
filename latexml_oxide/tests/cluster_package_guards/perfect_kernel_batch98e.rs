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
//! (`emitter/plan.rs` `cut_block_lines`); round 2 (review rev98e): a mark on a spacing nucleus leads its line
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

/// The fail-safe fires only for a block of two or more lines led by labels the names request (review rev98e S2): lines
/// led by a mark in `\( … \)` are read by the shared reader of a line's leading label (`relocate::spaced_mark_at`), each
/// author its line, as `$^{1}$` lines are (review N1's shape; the coordinator's 2026-11-01 directive: one reader, not a
/// fallback to the observer); marks inside one line's text (`Joint Institute, \textsuperscript{1}Univ A and
/// \textsuperscript{2}Univ B, City`) are no such block, but the group's one entry, a copy on each author (97b, 98a).
#[test]
fn the_fail_safe_fires_only_for_labelled_lines() {
  // (P4c: the lists' printed marks shown in the pins below, as pdflatex prints them — P4S Q1)
  let xml = both_ways(repro!("revtex_unread_mark_lines_go_by_their_marks"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "affiliation=1 Univ A, Town"],
    vec!["Bob Baker 2", "affiliation=2 Univ B, City"],
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
  // (P4c: the lists' printed marks shown in the pins below, as pdflatex prints them — P4S Q1)
  let xml = both_ways(repro!("revtex_nucleus_marks_lead_their_lines"));
  assert_eq!(creators(&xml), [
    // (each e-mail after the departments: REVTeX prints `\email` in the front matter's footnotes, its print zone the
    // footer's — 99a r3, OXIDIZED_DESIGN_DIVERGENCES #954)
    vec![
      "Ann Able 1 , 2",
      "affiliation=1 Department of Mathematics",
      "affiliation=and 2 Department of Physics, The Univ, City, USA",
      "email=able@x.edu",
    ],
    vec![
      "Bob Baker 1",
      "affiliation=1 Department of Mathematics",
      "email=baker@x.edu",
    ],
  ]);
  let xml = both_ways(repro!("revtex_boxed_nucleus_marks_lead_their_lines"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "affiliation=1 Univ A, Town"],
    vec!["Bob Baker 2", "affiliation=2 Univ B, City"],
  ]);
  // (the script braced whole, `${^1}$`: 2208.09667's lines after its first)
  let xml = both_ways(repro!("revtex_braced_script_marks_lead_their_lines"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "affiliation=1 Univ A, Town"],
    vec!["Bob Baker 2", "affiliation=2 Univ B, City"],
  ]);
  // (spacing unbraced, `$~^{1}$`: 1211.5032)
  let xml = both_ways(repro!("revtex_tie_nucleus_marks_lead_their_lines"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "affiliation=1 Univ A, Town"],
    vec![
      "Bob Baker 1,2",
      "affiliation=1 Univ A, Town",
      "affiliation=2 Univ B, City",
    ],
  ]);
}

/// The words of a line no name requests (an orphan mark's) print once in the document, on the block's creator — the
/// creator with no name — and on no author (user ruling 2026-11-05: the orphan's mark need not be kept, its words must,
/// never lost and never an author's).
#[track_caller]
fn once_on_the_block(xml: &str, words: &str) {
  assert_eq!(xml.matches(words).count(), 1, "{words:?} once in {xml}");
  let holders: Vec<Vec<String>> = creators(xml)
    .into_iter()
    .filter(|creator| creator.iter().any(|item| item.contains(words)))
    .collect();
  assert_eq!(holders.len(), 1, "{words:?} on one creator: {holders:?}");
  assert!(
    holders[0].iter().all(|item| item.contains('=')),
    "{words:?} on the block's creator, which has no name: {holders:?}"
  );
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
  // (the line kept its printed `b` while the writer cut it — review rev98e S1; since 107b the kernel's reader gives it
  // to the block as its shared orphans are, no mark shown, and its words once on the block's creator: user ruling
  // 2026-11-05, [`once_on_the_block`])
  assert_eq!(creators(&xml), [a.clone(), b.clone(), c.clone(), vec![
    "affiliation=Also at Eps College, Bergen"
  ],]);
  once_on_the_block(&xml, "Also at Eps College, Bergen");
  let xml = both_ways(repro!("revtex_block_lines_all_answered"));
  assert_eq!(creators(&xml), [a.clone(), b.clone(), c.clone()]);
  let xml = both_ways(repro!("revtex_block_lines_every_label_requested"));
  let mut c_both = c;
  c_both.push("affiliation=Also at Eps College, Bergen");
  assert_eq!(creators(&xml), [a, b, c_both]);
}
