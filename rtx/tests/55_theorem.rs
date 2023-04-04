///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use rustc_hash::FxHashMap as HashMap;

#[test]
#[ignore]
fn can_theorem() {
  let mut requires = HashMap::default();
  requires.insert("ntheorem", "ntheorem.std");
  rtx_tests("tests/theorem", Some(requires), None);
}
