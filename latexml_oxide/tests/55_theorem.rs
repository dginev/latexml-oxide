///**********************************************************************
/// Test cases for latexml_oxide — theorem tests
///**********************************************************************
use latexml::util::test::*;

use phf::phf_map;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
  "ntheorem" => "ntheorem.std",
  "ntheoremstyle" => "ntheorem.std",
};

const DIR: &str = "tests/theorem";

#[test]
fn amstheorem_test() {
  latexml_test_single("tests/theorem/amstheorem.tex", "amstheorem", DIR, Some(&REQUIRES), None);
}

#[test]
fn latextheorem_test() {
  latexml_test_single("tests/theorem/latextheorem.tex", "latextheorem", DIR, Some(&REQUIRES), None);
}

#[test]
#[ignore] // text= + tags diffs: math parser + equation numbering
fn ntheorem_test() {
  latexml_test_single("tests/theorem/ntheorem.tex", "ntheorem", DIR, Some(&REQUIRES), None);
}

#[test]
fn ntheoremstyle_test() {
  latexml_test_single("tests/theorem/ntheoremstyle.tex", "ntheoremstyle", DIR, Some(&REQUIRES), None);
}

#[test]
fn theorem_test() {
  latexml_test_single("tests/theorem/theorem.tex", "theorem", DIR, Some(&REQUIRES), None);
}
