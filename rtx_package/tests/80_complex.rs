#[macro_use]
pub extern crate rtx_codegen;
#[macro_use]
pub extern crate rtx_package;

mod helpers;

use rtx_core::common::error::*;
use rtx_core::state::State;
use rtx_core::stomach::Stomach;
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx_package::util::test::*;
use std::rc::Rc;

#[test]
fn can_complex() {
  let requires = None;
  rtx_tests_internal("tests/complex", requires, Some(Rc::new(complex_tests_dispatch)));
}

#[test]
#[ignore]
fn can_complex_todo() { rtx_tests("tests/complex_todo", None); }

pub fn complex_tests_dispatch(filename: &str, state: &mut State, stomach: Option<&mut Stomach>) -> Option<Result<()>> {
  match filename {
    // II. Connect the filename to the `load_definitions` function of your .rs binding:
    "xii.tex" => Some(helpers::xii_tex::load_definitions(state, stomach)),
    _ => None,
  }
}
