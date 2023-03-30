///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use rustc_hash::{FxHashMap as HashMap};

#[test]
#[ignore]
fn can_graphics() {
  let mut requires = HashMap::default();
  requires.insert("colors", "dvipsnam.def");
  requires.insert("xcolors", "dvipsnam.def");
  rtx_tests("tests/graphics", Some(requires), None);
}
