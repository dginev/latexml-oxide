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
use std::sync::Arc;

use rtx_core::stomach::Stomach;
use rtx_core::state::State;
use rtx_core::common::error::Result;
use rtx_package::package;
use rtx::util::test::*;

#[test]
fn can_digest() {
  rtx_tests("tests/digestion", None, Some(Arc::new(digestion_tests_dispatch)));
}

fn digestion_tests_dispatch(filename: &str, stomach: &mut Stomach, state: &mut State) -> Option<Result<()>> {
  match filename {
    "rebox.tex" => Some(helpers::rebox_tex::load_definitions(stomach, state)),
    other => package::dispatch(other, stomach, state),
  }
}
