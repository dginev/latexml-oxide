use phf::phf_map;
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use latexml::util::test::*;

static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
  "listing" => "listings.cfg"
};

#[test]
#[ignore]
fn can_align() { latexml_tests("tests/alignment", Some(&REQUIRES), None); }
