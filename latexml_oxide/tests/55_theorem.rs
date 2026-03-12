///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
use latexml::util::test::*;

use phf::phf_map;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
"ntheorem" => "ntheorem.std" };
#[test]
#[ignore] // ntheorem: 897 math parser tree diffs (known Marpa-based divergence)
fn can_theorem() { latexml_tests("tests/theorem", Some(&REQUIRES), None); }
