///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use rustc_hash::{FxHashMap as HashMap};

#[test]
#[ignore]
fn can_babel() {
  let mut requires = HashMap::default();
  requires.insert("*", "babel.sty");
  requires.insert("numprints", "numprint.sty");
  requires.insert("german", "germanb.ldf");
  requires.insert("greek", "greek.ldf");
  requires.insert("french", "frenchb.ldf");
  requires.insert("page545", "germanb.ldf");

  rtx_tests("tests/babel", Some(requires), None);
}
