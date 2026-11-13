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

use latexml::{
  tex_tests,
  util::test::{RequirementTable, Requirements},
};
use latexml_core::common::error::Result;

// Perl t/10_expansion.t.
static REQUIRES: &RequirementTable = &[
  ("meaning", Requirements::packages(&["t1enc.def"])),
  ("ifthen", Requirements::packages(&["ifthen.sty"])),
  ("textcase", Requirements::packages(&["textcase.sty"])),
  ("etoolbox", Requirements::packages(&["etoolbox.sty"])),
];

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
  Some(REQUIRES),
  Some(Rc::new(expansion_tests_dispatch))
);
