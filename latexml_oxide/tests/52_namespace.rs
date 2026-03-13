// Namespace tests — all ignored: DTD not supported in Rust port (only RelaxNG schemas)
use latexml::util::test::*;

const DIR: &str = "tests/namespace";

#[test]
#[ignore] // DTD not supported in Rust port
fn ns1_test() {
  latexml_test_single("tests/namespace/ns1.tex", "ns1", DIR, None, None);
}

#[test]
#[ignore] // DTD not supported in Rust port
fn ns2_test() {
  latexml_test_single("tests/namespace/ns2.tex", "ns2", DIR, None, None);
}

#[test]
#[ignore] // DTD not supported in Rust port
fn ns3_test() {
  latexml_test_single("tests/namespace/ns3.tex", "ns3", DIR, None, None);
}

#[test]
#[ignore] // DTD not supported in Rust port
fn ns4_test() {
  latexml_test_single("tests/namespace/ns4.tex", "ns4", DIR, None, None);
}

#[test]
#[ignore] // DTD not supported in Rust port
fn ns5_test() {
  latexml_test_single("tests/namespace/ns5.tex", "ns5", DIR, None, None);
}
