//! Red/green guards for batch 86c: two bindings that lost or misplaced printed title-page words. revtex4_support's
//! `\affil`, which no REVTeX class defines, is a guess a document's `\newcommand`/`\providecommand` replaces
//! (KNOWN_PERL_ERRORS #1199), and stays the class's under AASTeX, which defines it. The fcs and dgruyter bindings read
//! `\author*[labels]{Name}`: the star the corresponding author, the option the labels of the affiliations the name
//! requests (OXIDIZED_DESIGN_DIVERGENCES #905). Witnesses 1104.4073, 1907.12838, 1908.05079 (revtex), 2410.08723
//! (fcs), 1910.07269 (dgruyter). Each guard converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/`
//! (a minimal stand-in of the class or style, written by `{filecontents*}`, where the file is not in TeX Live) and
//! asserts no Error or Warning and every creator's name and contacts, in order. One guard per path: the document's
//! `\affil` (`\newcommand` and `\providecommand`); AASTeX's own `\affil` (the guess's boundary); fcs's starred
//! `\author` with `\address` labels; dgruyter's over a raw copy of its style.
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// Converts `tex`, checks it converts clean and has exactly `expected` creators (name, then each contact as
/// `role=text`), and returns the XML.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&[&str]]) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{xml}");
  xml
}

/// The document's `\affil` under REVTeX 4 (1104.4073, 1907.12838, 1908.05079): its `\newcommand{\affil}{…}` is the
/// macro `\affiliation{\affil}` prints, and so is a `\providecommand{\affil}[1]{…}`. Red (int116a): the `\newcommand`
/// ignored and the affiliation empty; the provided macro's argument alone as the affiliation.
#[test]
fn a_revtex_document_defines_its_own_affil() {
  assert_creators(repro!("revtex_document_newcommand_affil.tex"), &[&[
    "Martin Frimmer",
    "affiliation=Center for Nanophotonics, FOM Institute, Amsterdam",
  ]]);
  assert_creators(repro!("revtex_document_providecommand_affil.tex"), &[&[
    "Ann Able",
    "affiliation=Dept. of Physics, Univ. X",
  ]]);
}

/// The guess's boundary: AASTeX defines `\affil` (aastex701.cls:13186), so a document's `\providecommand{\affil}` leaves
/// the class's, whose call is the author's affiliation.
#[test]
fn aastex_keeps_its_own_affil() {
  assert_creators(repro!("aastex_class_affil_stays_the_class.tex"), &[&[
    "Ann Able",
    "affiliation=Univ. Y",
  ]]);
}

/// fcs's `\author*[1,2]{…}` (2410.08723): the starred name is the corresponding author (its envelope mark), each name
/// has the `\address[n]` lines its option lists, and a name without one has none. Red (int116a): a creator "*", the
/// name a paragraph after the abstract, both addresses the last author's.
#[test]
fn fcs_author_star_and_address_labels() {
  let xml = assert_creators(repro!("fcs_author_star_and_address_labels.tex"), &[
    &["Ann Author", "address=University of Witness"],
    &[
      "Bob Corresponding",
      "corresponding=\u{2709}",
      "address=University of Witness",
      "address=Institute of Repro",
    ],
    &["Carl Plain"],
  ]);
  assert!(!xml.contains("[1,2]"), "{xml}");
}

/// dgruyter's `\author*[1]{…}` over the raw copy of its style (1910.07269): the starred name is the corresponding
/// author, and each name has the `\affil[n]` its option lists. Red (int116a): a creator "*", "[1]Karoline Busse" a
/// paragraph after the abstract.
#[test]
fn dgruyter_author_star_and_affiliation_labels() {
  let xml = assert_creators(repro!("dgruyter_author_star_and_affil_labels.tex"), &[
    &[
      "Karoline Busse",
      "corresponding=Corresponding author",
      "affiliation=University of Bonn",
    ],
    &["Sabrina Amft", "affiliation=Leibniz University Hannover"],
    &["Daniel Hecker", "affiliation=University of Bonn"],
  ]);
  assert!(!xml.contains("[1]"), "{xml}");
}
