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
use rtx_package::package;

use phf::phf_map;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
"meaning" => "t1enc.def",
"ifthen" => "ifthen.sty"};

pub fn expansion_tests_dispatch(
  filename: &str,
  ) -> Option<Result<()>> {
  match filename {
    "whichinput.tex" => Some(helpers::whichinput_tex::load_definitions()),
    "whichcache.tex" => Some(helpers::whichcache_tex::load_definitions()),
    other => package::dispatch(other),
  }
}

tex_tests!(
  "tests/expansion",
  Some(&REQUIRES),
  Some(Rc::new(expansion_tests_dispatch))
);
