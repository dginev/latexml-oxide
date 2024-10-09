///**********************************************************************
/// Test cases for rtx
///**********************************************************************
#[macro_use]
extern crate latexml_codegen;
#[macro_use]
extern crate latexml_package;

mod helpers;

///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use std::rc::Rc;

use latexml::tex_tests;
use latexml_core::common::error::Result;

fn digestion_tests_dispatch(
  filename: &str,
  ) -> Option<Result<()>> {
  match filename {
    "rebox.tex" => Some(helpers::rebox_tex::load_definitions()),
    _ => None
  }
}

tex_tests!(
  "tests/digestion",
  None,
  Some(Rc::new(digestion_tests_dispatch))
);
