///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
use phf::phf_map;
use latexml::tex_tests;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
"csquotes" => "csquotes.sty" };

tex_tests!("tests/structure", Some(&REQUIRES), None);
