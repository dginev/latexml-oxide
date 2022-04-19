///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use std::collections::HashMap;

#[test]
#[ignore]
fn can_align() {
  let mut requires = HashMap::new();
  requires.insert("listing", "listings.cfg");
  rtx_tests("tests/alignment", Some(requires), None);
}
