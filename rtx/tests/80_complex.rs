#[macro_use]
extern crate rtx_codegen;
#[macro_use]
extern crate rtx_package;

mod helpers;

///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use rtx_core::common::error::*;
use rtx_core::state::State;
use rtx_core::stomach::Stomach;
use std::sync::Arc;

#[test]
fn can_complex() {
  let requires = None;
  rtx_tests_internal("tests/complex", requires, Some(Arc::new(complex_tests_dispatch)));
}

pub fn complex_tests_dispatch(filename: &str, stomach: &mut Stomach, state: &mut State) -> Option<Result<()>> {
  match filename {
    // II. Connect the filename to the `load_definitions` function of your .rs binding:
    "xii.tex" => Some(helpers::xii_tex::load_definitions(stomach, state)),
    _ => None,
  }
}
