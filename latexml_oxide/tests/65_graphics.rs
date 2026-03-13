///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
use latexml::util::test::*;
use phf::phf_map;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
"colors" => "dvipsnam.def",
"xcolors" => "dvipsnam.def" };

#[test]
#[ignore] // diffs — color.sty recursion
fn can_graphics() { latexml_tests("tests/graphics", Some(&REQUIRES), None); }
