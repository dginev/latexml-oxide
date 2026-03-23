// Math tests — individually listed for per-test #[ignore] support.
#[macro_use]
extern crate latexml_codegen;
#[macro_use]
extern crate latexml_package;

mod helpers;

use std::rc::Rc;

use latexml::util::test::*;
use latexml_core::common::error::Result;

const DIR: &str = "tests/math";

/// Dispatcher for math test source-level bindings (*_src.rs)
pub fn math_tests_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    "simplemath.latexml" => Some(helpers::simplemath_src::load_definitions()),
    _ => latexml_contrib::dispatch(filename),
  }
}

#[test]
fn ambiguous_relations_test() {
  latexml_test_single("tests/math/ambiguous_relations.tex", "ambiguous_relations", DIR, None, None);
}

#[test]
fn array_math_test() {
  latexml_test_single("tests/math/array.tex", "array", DIR, None, None);
}

#[test]
fn array_newline_math_test() {
  latexml_test_single("tests/math/array_newline_math.tex", "array_newline_math", DIR, None, None);
}

#[test]
fn arrows_test() {
  latexml_test_single("tests/math/arrows.tex", "arrows", DIR, None, None);
}

#[test]
fn choose_test() {
  latexml_test_single("tests/math/choose.tex", "choose", DIR, None, None);
}

#[test]
fn compact_dual_test() {
  latexml_test_single("tests/math/compact_dual.tex", "compact_dual", DIR, None, None);
}

#[test]
fn declare_test() {
  latexml_test_single("tests/math/declare.tex", "declare", DIR, None, None);
}

#[test]
fn fracs_test() {
  latexml_test_single("tests/math/fracs.tex", "fracs", DIR, None, None);
}

#[test]
fn niceunits_test() {
  latexml_test_single("tests/math/niceunits.tex", "niceunits", DIR, None, None);
}

#[test]
fn not_test() {
  latexml_test_single("tests/math/not.tex", "not", DIR, None, None);
}

#[test]
fn sampler_test() {
  latexml_test_single("tests/math/sampler.tex", "sampler", DIR, None, None);
}

#[test]
fn simplemath_test() {
  latexml_test_single("tests/math/simplemath.tex", "simplemath", DIR, None, Some(Rc::new(math_tests_dispatch)));
}

#[test]
fn testover_test() {
  latexml_test_single("tests/math/testover.tex", "testover", DIR, None, None);
}

#[test]
fn testscripts_test() {
  latexml_test_single("tests/math/testscripts.tex", "testscripts", DIR, None, None);
}
