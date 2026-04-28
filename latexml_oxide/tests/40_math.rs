// Math tests — one #[test] fn per `tests/math/*.tex+.xml` pair,
// generated at compile time by `tex_tests!`.
//
// One test (`simplemath`) loads definitions from the source-level
// helper `helpers::simplemath_src::load_definitions` via the
// `math_tests_dispatch` custom dispatcher; the rest fall through to
// `latexml_contrib::dispatch`, which is a strict no-op for files it
// doesn't recognise, so applying the same dispatcher directory-wide
// is safe.
#[macro_use]
extern crate latexml_engine;
#[macro_use]
extern crate latexml_codegen;
extern crate latexml_package;

mod helpers;

use std::rc::Rc;

use latexml::tex_tests;
use latexml_core::common::error::Result;

/// Dispatcher for math test source-level bindings (*_src.rs).
/// Handles `simplemath.latexml` for the `simplemath` test; all other
/// files fall through to the `latexml_contrib` dispatcher.
pub fn math_tests_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    "simplemath.latexml" => Some(helpers::simplemath_src::load_definitions()),
    _ => latexml_contrib::dispatch(filename),
  }
}

tex_tests!("tests/math", None, Some(Rc::new(math_tests_dispatch)));
