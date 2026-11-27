//! Guards for batch TEAMAUTH (2026-11-27, DG's ruling of that day): a team, group or collaboration the author line
//! enumerates — after a comma, "and" or "and the", or an `\and` column that is its name alone — is an author as pdflatex
//! prints it (`resolver::lines` `reads_as_listed_collaboration`, `resolver::blocks` `shared_column`; OXIDIZED_DESIGN_DIVERGENCES
//! #459). Two code paths, one test each with its controls: the names line (`an_enumerated_team_is_an_author`) and what keeps
//! its reading (`a_framed_collaboration_and_an_institution_column_keep_their_reading`); a group alone stays the creator
//! (`a_collaboration_alone_is_the_creator`).

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// `tex` converted: its (Error, Warning) counts, each creator's open tag, and each creator as [`creators`] reads it.
fn read(tex: &str) -> ((usize, usize), Vec<String>, Vec<Vec<String>>) {
  let (log, xml) = run(tex);
  let tags = xml
    .match_indices("<creator ")
    .map(|(at, _)| xml[at..at + xml[at..].find('>').expect("an open tag")].to_string())
    .collect();
  (
    (error_count(&log), warning_count(&log)),
    tags,
    creators(&xml),
  )
}

/// `tex` with its `\author{…}` argument `author` (the repro headers quote the line, so it is replaced by its command).
fn with_author(tex: &str, author: &str) -> String {
  let at = tex.find("\\author{").expect("an author line");
  let end = at + tex[at..].find('\n').expect("its line end");
  let out = format!("{}\\author{{{author}}}{}", &tex[..at], &tex[end..]);
  assert!(out.contains(&format!("\\author{{{author}}}\n")), "{out}");
  out
}

/// The rows whose (Error, Warning) counts, creators or creator tags differ from the expected: all of them, so one run
/// shows every row's state.
/// A row: its name, the document, its (Error, Warning) counts and its creators as [`creators`] reads them.
type TeamRow<'a> = (&'a str, String, (usize, usize), Vec<Vec<String>>);

fn mismatches(rows: &[TeamRow]) -> Vec<String> {
  rows
    .iter()
    .filter_map(|(row, tex, counts, expected)| {
      let (got_counts, tags, got) = read(tex);
      let roles =
        tags.len() == expected.len() && tags.iter().all(|t| t.contains("role=\"author\""));
      (got_counts != *counts || got != *expected || !roles).then(|| {
        format!("{row}: got {got_counts:?} {got:?} {tags:?}, expected {counts:?} {expected:?}")
      })
    })
    .collect()
}

fn names(names: &[&str]) -> Vec<Vec<String>> { names.iter().map(|n| vec![n.to_string()]).collect() }

/// Each enumerated team is a creator `role="author"`, its text the personname, and no name carries a collaboration or an
/// affiliation contact: `… and R Core Team` and its `\and` form (the R project's author line; Perl: three creators), `and
/// the GAPS Team` (1708.04166, 1609.09534's `and the MAST Team`), `and LHD experimental group` (1711.07097). rc6: two
/// creators, the team a `collaboration` contact on each (the `\and` form an `affiliation`; after a list's comma, the
/// affiliation of the name before it — one name's comma keeps that reading, `Bob Baker, \TeX{} Users Group`,
/// `perfect_kernel_batchlogo1`). llncs's inline `\and` too (its `the ATLAS Collaboration`:
/// `perfect_kernel_batch77t::llncs_author_and_piece_is_names`).
#[test]
fn an_enumerated_team_is_an_author() {
  let r_core = repro!("author_enumerated_team_is_an_author");
  let gaps = repro!("author_enumerated_the_team_is_an_author");
  let r_core_names = names(&["Friedrich Leisch", "Vincent Goulet", "R Core Team"]);
  let rows = [
    (
      "comma and",
      r_core.to_string(),
      (0, 0),
      r_core_names.clone(),
    ),
    (
      "\\and column",
      with_author(
        r_core,
        "Friedrich Leisch \\and Vincent Goulet \\and R Core Team",
      ),
      (0, 0),
      r_core_names,
    ),
    (
      "and the",
      gaps.to_string(),
      (0, 0),
      names(&["A. Author", "B. Author", "the GAPS Team"]),
    ),
    (
      "and, lowercase",
      with_author(gaps, "A. Author and LHD experimental group"),
      (0, 0),
      names(&["A. Author", "LHD experimental group"]),
    ),
    (
      "a list's comma",
      with_author(gaps, "A. Author, B. Author, R Core Team"),
      (0, 0),
      names(&["A. Author", "B. Author", "R Core Team"]),
    ),
    (
      "llncs's inline \\and",
      with_author(gaps, "A. Author \\and B. Author \\and R Core Team").replacen(
        "{article}",
        "{llncs}",
        1,
      ),
      (0, 0),
      names(&["A. Author", "B. Author", "R Core Team"]),
    ),
  ];
  let bad = mismatches(&rows);
  assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// A collaboration that is the line's only name is its creator, as rc6 reads it: the enumeration rule needs a person
/// before the group, so it never leaves a block without a creator (a red-team probe of the rule).
#[test]
fn a_collaboration_alone_is_the_creator() {
  let tex = with_author(
    repro!("author_enumerated_team_is_an_author"),
    "The IceCube Collaboration",
  );
  let rows = [("alone", tex, (0, 0), names(&["The IceCube Collaboration"]))];
  let bad = mismatches(&rows);
  assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// What the author line does not enumerate keeps its reading: a name's "for" remark is a `collaboration` contact on the
/// names it covers (its two Warnings the emitter's markup fallback, `author_names_inline_remark_markup_fallback_red`), a
/// bracketed line under the names too (2410.11607's shape); so is a group after a remark open on the line (1311.7305,
/// hep-ex0611001, 2009.04166; TEAMAUTH's first build made "the Y Team" a creator) and a plural joining two groups' names
/// (today's split pinned, RED in `author_collaboration_name_with_and_stays_one`; the first build moved "CERN" to a
/// nameless creator); an institution's `\and` column, `The R Foundation` (DG 2026-11-27: ambiguous, today's reading), is
/// every name's affiliation.
#[test]
fn a_framed_collaboration_and_an_institution_column_keep_their_reading() {
  let gaps = repro!("author_enumerated_the_team_is_an_author");
  let with = |name: &str, contact: &str| vec![name.to_string(), contact.to_string()];
  let atlas = "collaboration=for the ATLAS Collaboration";
  let r_foundation = "affiliation=The R Foundation";
  let with2 =
    |name: &str, one: &str, two: &str| vec![name.to_string(), one.to_string(), two.to_string()];
  let (x_team, y_team) = (
    "collaboration=representing the X Team",
    "collaboration=the Y Team",
  );
  let rows = [
    (
      "for remark",
      with_author(gaps, "A. Author and B. Author for the ATLAS Collaboration"),
      (0, 2),
      vec![with("A. Author", atlas), with("B. Author", atlas)],
    ),
    (
      "bracketed line",
      with_author(gaps, "A. Author \\\\ (XYZ Collaboration)"),
      (0, 0),
      vec![with("A. Author", "collaboration=(XYZ Collaboration)")],
    ),
    (
      "a remark open on the line",
      with_author(
        gaps,
        "Ann Able and Bob Baker, representing the X Team and the Y Team",
      ),
      (0, 0),
      vec![
        with2("Ann Able", x_team, y_team),
        with2("Bob Baker", x_team, y_team),
      ],
    ),
    (
      "a plural joining two groups",
      repro!("author_collaboration_name_with_and_stays_one").to_string(),
      (0, 0),
      vec![with2(
        "The ATLAS",
        "collaboration=CMS Collaborations",
        "affiliation=CERN",
      )],
    ),
    (
      "institution column",
      with_author(
        repro!("author_enumerated_team_is_an_author"),
        "Friedrich Leisch \\and Vincent Goulet \\and The R Foundation",
      ),
      (0, 0),
      vec![
        with("Friedrich Leisch", r_foundation),
        with("Vincent Goulet", r_foundation),
      ],
    ),
  ];
  let bad = mismatches(&rows);
  assert!(bad.is_empty(), "{}", bad.join("\n"));
}
