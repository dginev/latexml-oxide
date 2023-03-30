use rtx::util::test::*;
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rustc_hash::{FxHashMap as HashMap};

#[test]
fn can_structure() {
  let mut requires = HashMap::default();
  requires.insert("csquotes", "csquotes.sty");
  rtx_tests("tests/structure", Some(requires), None);
}
