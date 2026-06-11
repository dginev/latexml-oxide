///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
#[macro_use]
extern crate latexml_engine;
#[macro_use]
extern crate latexml_codegen;
extern crate latexml_package;

mod helpers;

use std::rc::Rc;

use latexml::tex_tests;
use latexml_core::common::error::Result;

fn grouping_tests_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    "scopemacro.latexml" => Some(helpers::scopemacro_src::load_definitions()),
    _ => latexml_contrib::dispatch(filename),
  }
}

tex_tests!(
  "tests/grouping",
  None,
  Some(Rc::new(grouping_tests_dispatch))
);
