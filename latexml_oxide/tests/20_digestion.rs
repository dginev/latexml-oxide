///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
#[macro_use]
extern crate latexml_engine;
#[macro_use]
extern crate latexml_codegen;
extern crate latexml_package;

mod helpers;

///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
use std::rc::Rc;

use latexml::tex_tests;
use latexml_core::common::error::Result;

fn digestion_tests_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    // Document-level binding: loaded by load_external_binding(stem)
    "rebox.latexml" => Some(helpers::rebox_src::load_definitions()),
    _ => None,
  }
}

tex_tests!(
  "tests/digestion",
  None,
  Some(Rc::new(digestion_tests_dispatch))
);
