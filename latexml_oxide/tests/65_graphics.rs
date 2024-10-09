use phf::phf_map;
///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
use latexml::util::test::*;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
"colors" => "dvipsnam.def",
"xcolors" => "dvipsnam.def" };

#[test]
#[ignore]
fn can_graphics() { latexml_tests("tests/graphics", Some(&REQUIRES), None); }
