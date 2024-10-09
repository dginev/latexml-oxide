#[macro_use]
extern crate latexml_codegen;
#[macro_use]
extern crate latexml_package;

mod helpers;

///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use latexml::util::test::*;
use latexml_core::common::error::*;
use std::rc::Rc;

#[test]
fn can_complex() {
  let requires = None;
  latexml_tests_internal(
    "tests/complex",
    requires,
    Some(Rc::new(complex_tests_dispatch)),
  );
}

pub fn complex_tests_dispatch(
  filename: &str,
  ) -> Option<Result<()>> {
  match filename {
    // II. Connect the filename to the `load_definitions` function of your .rs binding:
    "xii.tex" => Some(helpers::xii_tex::load_definitions()),
    _ => None,
  }
}
