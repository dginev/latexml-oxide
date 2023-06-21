///**********************************************************************
/// Test cases for rtx
///**********************************************************************
#[macro_use]
extern crate rtx_codegen;
#[macro_use]
extern crate rtx_package;

mod helpers;

///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use std::rc::Rc;

use rtx::tex_tests;
use rtx_core::common::error::Result;
use rtx_core::{state_mut,state};
use rtx_core::stomach::Stomach;
use rtx_package::package;

fn digestion_tests_dispatch(
  filename: &str,
  ) -> Option<Result<()>> {
  match filename {
    "rebox.tex" => Some(helpers::rebox_tex::load_definitions(stomach)),
    other => package::dispatch(other),
  }
}

tex_tests!(
  "tests/digestion",
  None,
  Some(Rc::new(digestion_tests_dispatch))
);
