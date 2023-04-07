#[macro_use]
extern crate rtx_codegen;
#[macro_use]
extern crate rtx_package;

mod helpers;

///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use std::sync::Arc;

use rtx_core::common::error::Result;
use rtx_core::state::State;
use rtx_core::stomach::Stomach;
use rtx_package::package;
use rtx::tex_tests;

use phf::{phf_map};
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
  "meaning" => "t1enc.def",
  "ifthen" => "ifthen.sty"};

pub fn expansion_tests_dispatch(
  filename: &str,
  stomach: &mut Stomach,
  state: &mut State,
) -> Option<Result<()>> {
  match filename {
    "whichinput.tex" => Some(helpers::whichinput_tex::load_definitions(stomach, state)),
    "whichcache.tex" => Some(helpers::whichcache_tex::load_definitions(stomach, state)),
    other => package::dispatch(other, stomach, state),
  }
}

tex_tests!(
  "tests/expansion",
  Some(&REQUIRES),
  Some(Arc::new(expansion_tests_dispatch)));
