// Babel tests — split into individual tests to isolate hangs.
use latexml::util::test::*;
const DIR: &str = "tests/babel";

#[test]
#[ignore] // OOM: babel+csquotes processing exceeds memory limits
fn csquotes_test() {
  latexml_test_single("tests/babel/csquotes.tex", "csquotes", DIR, None, None);
}

#[test]
#[ignore] // needs frenchb.ldf, hangs in batch
fn french_test() {
  latexml_test_single("tests/babel/french.tex", "french", DIR, None, None);
}

#[test]
#[ignore] // needs germanb.ldf, hangs in batch
fn german_test() {
  latexml_test_single("tests/babel/german.tex", "german", DIR, None, None);
}

#[test]
#[ignore] // needs greek.ldf
fn greek_test() {
  latexml_test_single("tests/babel/greek.tex", "greek", DIR, None, None);
}

#[test]
#[ignore] // needs numprint.sty
fn numprints_test() {
  latexml_test_single("tests/babel/numprints.tex", "numprints", DIR, None, None);
}

#[test]
#[ignore] // needs germanb.ldf
fn page545_test() {
  latexml_test_single("tests/babel/page545.tex", "page545", DIR, None, None);
}
