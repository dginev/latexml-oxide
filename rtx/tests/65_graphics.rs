use phf::phf_map;
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
"colors" => "dvipsnam.def",
"xcolors" => "dvipsnam.def" };

#[test]
#[ignore]
fn can_graphics() { rtx_tests("tests/graphics", Some(&REQUIRES), None); }
