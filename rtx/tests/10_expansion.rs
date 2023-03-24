#[macro_use]
extern crate rtx_codegen;
#[macro_use]
extern crate rtx_package;

mod helpers;

///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use std::sync::Arc;
use std::collections::HashMap;

use rtx_core::stomach::Stomach;
use rtx_core::state::State;
use rtx_core::common::error::Result;
use rtx_package::package;

use rtx::util::test::*;

#[test]
fn can_expand() {
  let mut requires = HashMap::new();
  requires.insert("meaning", "t1enc.def");
  requires.insert("ifthen", "ifthen.sty");

  rtx_tests("tests/expansion", Some(requires), Some(Arc::new(expansion_tests_dispatch)));
}
pub fn expansion_tests_dispatch(filename: &str, stomach: &mut Stomach, state: &mut State) -> Option<Result<()>> {
  match filename {
    "whichinput.tex" => Some(helpers::whichinput_tex::load_definitions(stomach, state)),
    "whichcache.tex" => Some(helpers::whichcache_tex::load_definitions(stomach, state)),
    other => package::dispatch(other, stomach, state),
  }
}
