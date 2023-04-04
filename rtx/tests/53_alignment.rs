///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use rustc_hash::FxHashMap as HashMap;

#[test]
#[ignore]
fn can_align() {
  let mut requires = HashMap::default();
  requires.insert("listing", "listings.cfg");
  rtx_tests("tests/alignment", Some(requires), None);
}
