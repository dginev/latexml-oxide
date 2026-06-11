///**********************************************************************
/// Test cases for latexml_oxide — theorem tests
///**********************************************************************
use latexml::tex_tests;
use phf::phf_map;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
  "ntheorem" => "ntheorem.std",
  "ntheoremstyle" => "ntheorem.std",
};

tex_tests!("tests/theorem", Some(&REQUIRES), None);
