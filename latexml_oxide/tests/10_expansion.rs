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
    _ => None
  }
}

tex_tests!(
  "tests/expansion",
  Some(&REQUIRES),
  Some(Rc::new(expansion_tests_dispatch))
);
