use rtx::tex_tests;
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use phf::{phf_map};
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
  "csquotes" => "csquotes.sty" };

tex_tests!("tests/structure", Some(&REQUIRES), None);