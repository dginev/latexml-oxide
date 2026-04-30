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

use phf::phf_map;
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
"meaning" => "t1enc.def",
"ifthen" => "ifthen.sty"};

pub fn expansion_tests_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    // Document-level bindings: loaded by load_external_binding(stem)
    // Translate the Perl .latexml files to Rust
    "whichinput.latexml" => Some(helpers::whichinput_tex::load_definitions()),
    "whichcache.latexml" => Some(helpers::whichcache_tex::load_definitions()),
    _ => None,
  }
}

tex_tests!(
  "tests/expansion",
  Some(&REQUIRES),
  Some(Rc::new(expansion_tests_dispatch))
);
