//! Forced-streaming sweep over `tests/complex`.
//!
//! It needs its own dispatcher: `xii.latexml` and `labelled.latexml` are TEST helpers,
//! not contrib entries, so the plain `sweep_dir` would convert those fixtures without
//! their bindings and fail with misdefined / undefined errors.

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

fn complex_tests_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    "xii" | "xii.latexml" => Some(helpers::xii_tex::load_definitions()),
    "labelled.latexml" => Some(helpers::labelled_tex::load_definitions()),
    _ => latexml_contrib::dispatch(filename),
  }
}

#[test]
fn streaming_matches_eager_on_complex() { sweep_dir_with("tests/complex", complex_tests_dispatch); }
