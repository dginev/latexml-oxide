//! Forced-streaming sweep over `tests/digestion`.
//!
//! It needs its own dispatcher: `rebox.latexml` is a TEST helper,
//! not a contrib entry, so the plain `sweep_dir` would convert that fixture without
//! its bindings and fail with undefined macro errors.

#[macro_use]
extern crate latexml_engine;
#[macro_use]
extern crate latexml_codegen;
extern crate latexml_contrib;
extern crate latexml_package;

mod helpers;
mod streaming_sweep;

use latexml_core::common::error::Result;
use streaming_sweep::sweep_dir_with;

fn digestion_tests_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    "rebox.latexml" => Some(helpers::rebox_src::load_definitions()),
    _ => latexml_contrib::dispatch(filename),
  }
}

#[test]
fn streaming_matches_eager_on_digestion() {
  sweep_dir_with("tests/digestion", digestion_tests_dispatch);
}
