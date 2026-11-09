//! Guards for FB2b2, two shapes of the old reader's names split ported onto the one engine's reading
//! (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`; FB1's port P2), each fixture converted with the emitter's switch
//! off and at `LATEXML_FM_EMIT=2`, the creators (name and every contact, in order) the same both ways: a byline's "By"
//! that a spacing run parts from the names (the reading's `byline_dropped` on a names line, the old `without_leading_by`,
//! the content check counting the word it drops as a separator's), and a line under the names opening with "and" after
//! a space (`later_line_is_names`, the old `names_list_continues`).

use super::perfect_kernel_batchnl1::both;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A byline's "By" in a run of its own, the spacing or the font after it parting it from the names, is no name text
/// (`By~~{\scshape Vitaly~Bergelson}~~and~~…`, 2603.25922, 2604.07186, 2604.12864; `By~~{\scshape Bryna Kra}`,
/// 2603.27258), at the block's first line and at an `\and` column's (`\and BY~~Max Moss~~and~~Ned Nash`): the lexer
/// keeps it a Name, the reading drops it on a names line (`lines::byline_dropped`) and records it
/// (`Resolved::drops`), which the content check counts as a separator's word. Red (NL1d `=2`): the creators "By
/// Bryna Kra", "BY Max Moss", "By Ann Able"; (NL1e `=2`) a Warning "missing from the placed frontmatter: By".
#[test]
fn a_byline_word_alone_is_no_name() {
  let two = both(repro!("author_byline_by_before_spacing_runs"), 0);
  let want = [
    vec!["Bryna Kra"],
    vec!["Joel Moreira"],
    vec!["Florian K. Richter"],
    vec!["Donald Robertson"],
  ];
  assert_eq!(two, want);
  let two = both(repro!("author_and_word_in_any_case_or_wrapper"), 0);
  let want: Vec<Vec<&str>> = [
    "Ann Able",
    "Bob Baker",
    "Cat Cole",
    "Dan Dorn",
    "Eve Eads",
    "Fay Fox",
    "Gil Gray",
    "Hal Hunt",
    "Ivy Ives",
    "Jon Jay",
    "Kim Key",
    "Lou Lee",
    "Max Moss",
    "Ned Nash",
  ]
  .into_iter()
  .map(|name| vec![name])
  .collect();
  assert_eq!(two, want);
  // (pdflatex: "By Ann Able and Bob Baker" over "Univ A")
  let two = both(
    "\\documentclass{article}\n\\title{T}\n\\author{By~~Ann Able~~and~~Bob Baker \\\\ Univ A}\n\
     \\begin{document}\n\\maketitle\nText.\n\\end{document}\n",
    0,
  );
  let want = [vec!["Ann Able", "affiliation=Univ A"], vec![
    "Bob Baker",
    "affiliation=Univ A",
  ]];
  assert_eq!(two, want);
}

/// A line under the names that opens with "and" after a space goes on with the names: the old reader trims the line's
/// text before its `starts_with("and ")` (`names_list_continues`), so the space before the word opens nothing
/// (`\\~~and~~{\scshape Donald Robertson}`, 2603.27258, in the fixture above; `\\ \quad and Cat Cole`). Red (NL1d `=2`):
/// "and Cat Cole" an affiliation of both names above it, its markup lost (a Warning).
#[test]
fn an_and_line_after_a_space_goes_on_with_the_names() {
  // (pdflatex: "Ann Able and Bob Baker" over "and Cat Cole")
  let two = both(
    "\\documentclass{article}\n\\title{T}\n\\author{Ann Able and Bob Baker\\\\ \\quad and Cat \
     Cole}\n\\begin{document}\n\\maketitle\nText.\n\\end{document}\n",
    0,
  );
  let want = [vec!["Ann Able"], vec!["Bob Baker"], vec!["Cat Cole"]];
  assert_eq!(two, want);
}

/// A byline's "By" opening a line the reading reads as no names keeps its ink: `Ann Able \\ By\quad Lee Hall,
/// \textit{Univ X}` is Ann Able's affiliation, its italic kept (review revG1 1: the lexer's split reached affiliation
/// lines). Red (FB2b3 `=2`): "By" dropped, the italic lost, a Warning. Off reads b10's `\quad` line as names (the old
/// reader's split: an affiliation "By", the creators "Lee Hall" and "Univ X"), so only `=2` is pinned there; b11, no
/// spacing, reads the same both ways.
#[test]
fn a_byline_word_on_an_affiliation_line_keeps_its_ink() {
  let want = [vec!["Ann Able", "affiliation=By Lee Hall, Univ X"]];
  let italic = "Lee Hall, <text font=\"italic\"";
  // (pdflatex: "Ann Able" over "By Lee Hall, Univ X", both)
  for (spacing, _) in [("\\quad ", false), (" ", true)] {
    let tex = format!(
      "\\documentclass{{article}}\n\\begin{{document}}\n\\title{{T}}\\author{{Ann Able \\\\ By{spacing}Lee Hall, \
       \\textit{{Univ X}}}}\n\\maketitle\nText.\n\\end{{document}}\n"
    );
    let two = both(&tex, 0);
    assert_eq!(two, want, "{spacing}");
    let xml = super::perfect_kernel_batchz4::run(&tex).1;
    assert!(xml.contains(italic), "{xml}");
  }
}
