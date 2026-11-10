//! Guards for CR7: the emitter's footnote-counter and definition declines write the author block as Perl's
//! `\lx@add@authors` does (Base_Utility.pool.ltxml:676-720, `fallback::perl_author_calls`; OXIDIZED_DESIGN_DIVERGENCES
//! #988), split where Perl splits it and its pieces digested in order, not merged into one creator (Perl's tabular branch
//! only, which stays `whole_block_author`). One guard per path: each decline on the split branch (CR5's repros), the
//! marks branch, the split's `\\` lines keeping their commas (the divergence from Perl's comma), and each split name's
//! own label for the items the reading gave it. CR7b (review revCR67): a `\\[len]` cut leaves no "[len]" in a name, a line
//! of no mark cut at `\and` starts an author, and the labels match the reading's names to Perl's creators by name.

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

/// Every `<creator …>…</creator>` element of `xml`, whole, in document order, the indentation between its tags out.
fn creator_elements(xml: &str) -> Vec<String> {
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let end =
        start + xml[start..].find("</creator>").expect("closed creator") + "</creator>".len();
      xml[start..end].lines().map(str::trim).collect::<String>()
    })
    .collect()
}

/// `\author{Ann Able\thanks{First.} \and Bob Baker\stepcounter{footnote}\footnote{Work done at X.}}`: the counter's step
/// before Bob's note is no event of the reading, so the emitter declines it. It wrote one creator "Ann Able and Bob Baker"
/// holding both notes (Perl's tabular branch, the Warning's "as Perl's whole-block creator" false); Perl splits it at
/// `\and`, each piece digested in order, as pdflatex prints "Ann Able∗ Bob Baker‡": two creators, Ann's thanks note, Bob's
/// note, and the body's footnote 1 after the title's reset (1003.2281 6→1, cs0404029 6→1, 2410.22720 4→1).
#[test]
fn a_counter_decline_writes_the_block_as_perl_splits_it() {
  let (log, xml) = run(repro!("cr7_fncounter_and"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "The frontmatter emitter's reading holds no place for what its author block does to the footnote counter; the \
       block is written as Perl's \\lx@add@authors splits it at its separators, its pieces digested in order"
    ),
    "{log}"
  );
  assert_eq!(
    creator_elements(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able</personname><note class=\"ltx_note_frontmatter \
       ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">First.</note></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
       name=\"Note:\u{a0}\" role=\"note\">Work done at X.</contact></creator>",
    ],
    "{xml}"
  );
  assert!(
    xml.contains(r#"<note mark="1" role="footnote" xml:id="footnote1">"#),
    "{xml}"
  );
}

/// `\author{Ann Able\footremember{zz}{Zebra University.} \and Bob Baker}`, `\footremember` a note then `\newcounter`:
/// its declaration can't be replayed apart from what it prints, so the emitter declines the block; it wrote one creator
/// "Ann Able and Bob Baker". As Perl splits it: Ann with her note, Bob (pdflatex "Ann Able∗ Bob Baker"; 1404.5602 4→1,
/// 2503.21719 3→1, 0809.5152 5→1, 1110.6375 11→1).
#[test]
fn a_definition_decline_writes_the_block_as_perl_splits_it() {
  let (log, xml) = run(repro!("cr7_ne5_footremember"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "The frontmatter emitter can't replay a definition of its author block apart from what the block prints; the \
       block is written as Perl's \\lx@add@authors splits it at its separators, its pieces digested in order"
    ),
    "{log}"
  );
  assert_eq!(
    creator_elements(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Note:\u{a0}\" \
       role=\"note\">Zebra University.</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>",
    ],
    "{xml}"
  );
}

/// A declined block holding superscript marks takes Perl's marks branch: `Ann Able$^{1}$, Bob Baker$^{2}$\footremember…,
/// and Cat Cole$^{1}$\\ $^{1}$Univ X, Dept Y\\ $^{2}$Univ Z` — the names line cut at its commas and " and ", each marked
/// line an affiliation the names' marks request, as Perl writes it (1404.5602's shape).
#[test]
fn a_declined_block_with_marks_takes_perls_marks_branch() {
  let (log, xml) = run(repro!("cr7_marks_branch_footremember"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "the block is written as Perl's \\lx@add@authors splits a block with superscript marks"
    ),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "affiliation=Univ X, Dept Y"]),
    row(&["Bob Baker", "note=Zebra note.", "affiliation=Univ Z"]),
    row(&["Cat Cole", "affiliation=Univ X, Dept Y"]),
  ]);
}

/// Perl's split cuts at the comma before the `\\` lines, so `Ann Able\footremember…\\ Dept of Physics, Univ X, City \and
/// Bob Baker\\ Univ Y` read "Univ X" and "City" as authors (Perl: four creators); the declined block's split cuts only
/// a names line at the comma (OD #988; 1302.3028's PoS address line).
#[test]
fn a_split_keeps_the_commas_of_its_lines() {
  let (log, xml) = run(repro!("cr7_split_lines_keep_commas"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(creators(&xml), [
    row(&[
      "Ann Able",
      "note=Zebra note.",
      "affiliation=Dept of Physics, Univ X, City"
    ]),
    row(&["Bob Baker", "affiliation=Univ Y"]),
  ]);
}

/// A split whose creators are the names the reading read, one for one, gives each name its own label: revtex's
/// `\affiliation{Univ A}` after `\author{Ann Able\footremember… and Bob Baker}` goes to both (it went to Ann alone, the
/// labels of the block written whole), and the next `\author{Cat Cole}`'s `\affiliation{Univ C}` to Cat, past the split's
/// labels (synthetic: the creators loop's owner labels).
#[test]
fn a_split_gives_each_name_its_items() {
  let (log, xml) = run(repro!("cr7_split_items_to_each_name"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "note=Zebra note.", "affiliation=Univ A"]),
    row(&["Bob Baker", "affiliation=Univ A"]),
    row(&["Cat Cole", "affiliation=Univ C"]),
  ]);
}

/// `Ann Able$^{1}$\footremember…\\[1em] Bob Baker$^{1}$\\ $^{1}$Univ X`: the split cut at `\\` left "[1em]" heading Bob's
/// line, so the creator read "[1em] Bob Baker" with a content-check Warning (Perl alike, its own comment at
/// Base_Utility.pool.ltxml:670); pdflatex prints no "[1em]" (2007.01340's "[1em] Julia Fukuyama"; review revCR67 B1).
#[test]
fn a_split_at_a_linebreak_option_leaves_no_option_text() {
  let (log, xml) = run(repro!("cr7b_linebreak_option_cut"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(
    creator_elements(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Note:\u{a0}\" \
       role=\"note\">Zebra note.</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ \
       X</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
       name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact></creator>",
    ],
    "{xml}"
  );
}

/// `Ann Able$^{1}$\footremember… \and Bob Baker\\ $^{1}$Univ X`: Bob's line has no mark, and it continued Ann's entry with
/// its `\and`, one creator "Ann Able and Bob Baker" (Perl "Ann AbleBob Baker"); a cut at the `\and` family starts an author
/// (OD #52(g)), as pdflatex prints two (review revCR67 S1). Distinct from the `\\` continuation, which the marks-branch
/// guard keeps.
#[test]
fn a_line_of_no_mark_cut_at_and_starts_an_author() {
  let (log, xml) = run(repro!("cr7b_and_cut_starts_author"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(
    creator_elements(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Note:\u{a0}\" \
       role=\"note\">Zebra note.</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ \
       X</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>",
    ],
    "{xml}"
  );
}

/// `\author{Robert Heath, Jr.\footremember… and Ann Able~~~~Bob Baker}\email{bob@x.org}\affiliation{Univ A}`: Perl's three
/// creators "Robert Heath", "Jr.", "Ann Able~~~~Bob Baker" and the reading's three names "Robert Heath, Jr.", "Ann Able",
/// "Bob Baker" — matched by count, Ann's `\affiliation` went to "Jr."; matched by name, Ann and Bob's items go to the
/// creator holding them (review revCR67 S4). Distinct from `a_split_gives_each_name_its_items`: equal counts, other names.
#[test]
fn a_split_gives_each_name_its_items_by_name() {
  let (log, xml) = run(repro!("cr7b_split_items_by_name"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Robert Heath", "affiliation=Univ A"]),
    row(&["Jr.", "note=Zebra note."]),
    row(&[
      "Ann Able Bob Baker",
      "affiliation=Univ A",
      "email=bob@x.org"
    ]),
  ]);
}
