///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
use latexml::util::test::*;
use phf::phf_map;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
"*" => "babel.sty",
"numprints" => "numprint.sty",
"german" => "germanb.ldf",
"greek" => "greek.ldf",
"french" => "frenchb.ldf",
"page545" => "germanb.ldf"};

#[test]
#[ignore]
fn can_babel() { latexml_tests("tests/babel", Some(&REQUIRES), None); }
