///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use phf::{phf_map};

static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
  "listing" => "listings.cfg"
};

#[test]
#[ignore]
fn can_align() {
  rtx_tests("tests/alignment", Some(&REQUIRES), None);
}
