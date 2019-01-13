///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx_package::util::test::*;
use std::collections::HashMap;

#[test]
#[ignore]
fn can_babel() {
  let mut requires = HashMap::new();
  requires.insert("*", "babel.sty");
  requires.insert("numprints", "numprint.sty");
  requires.insert("german", "germanb.ldf");
  requires.insert("greek", "greek.ldf");
  requires.insert("french", "frenchb.ldf");
  requires.insert("page545", "germanb.ldf");

  rtx_tests("tests/babel", Some(requires));
}
