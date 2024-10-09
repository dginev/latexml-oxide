///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use latexml::util::test::*;

use phf::phf_map;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
"ntheorem" => "ntheorem.std" };
#[test]
#[ignore]
fn can_theorem() { latexml_tests("tests/theorem", Some(&REQUIRES), None); }
