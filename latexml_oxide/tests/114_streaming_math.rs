//! Forced-streaming corpus sweep — the wide version of `113_streaming_core`.
//! Every fixture converts twice (eager, and streaming with an aggressive
//! 3-box budget); the XML must be byte-identical and both conversions error-free.
//! One suite per test binary — see `streaming_sweep/mod.rs` for why.
//!
//! It needs the math suite's dispatcher (`40_math.rs`): `simplemath.latexml` is a TEST helper,
//! not a contrib entry, so the plain `sweep_dir` would read that Perl binding as TeX.

// `helpers` carries `LoadDefinitions!`-based binding sources, so it needs the
// same macro preamble the eager math suite (`40_math.rs`) has.
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

fn math_tests_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    "simplemath.latexml" => Some(helpers::simplemath_src::load_definitions()),
    _ => latexml_contrib::dispatch(filename),
  }
}

#[test]
fn streaming_matches_eager_on_math() { sweep_dir_with("tests/math", math_tests_dispatch); }
