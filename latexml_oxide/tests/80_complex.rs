#[macro_use]
extern crate latexml_codegen;
#[macro_use]
extern crate latexml_package;

mod helpers;

use latexml::util::test::*;
use latexml_core::common::error::*;
use std::rc::Rc;

const DIR: &str = "tests/complex";

pub fn complex_tests_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    "xii" => Some(helpers::xii_tex::load_definitions()),
    _ => None,
  }
}

#[test]
fn xii_test() {
  latexml_test_single(
    "tests/complex/xii.tex", "xii", DIR, None,
    Some(Rc::new(complex_tests_dispatch)),
  );
}

#[test]
#[ignore] // needs aastex631.cls binding
fn aastex631_deluxetable_test() {
  latexml_test_single(
    "tests/complex/aastex631_deluxetable.tex", "aastex631_deluxetable", DIR, None,
    Some(Rc::new(complex_tests_dispatch)),
  );
}

#[test]
#[ignore] // needs acmart.cls binding
fn acm_aria_test() {
  latexml_test_single(
    "tests/complex/acm_aria.tex", "acm_aria", DIR, None,
    Some(Rc::new(complex_tests_dispatch)),
  );
}

#[test]
fn figure_dual_caption_test() {
  latexml_test_single(
    "tests/complex/figure_dual_caption.tex", "figure_dual_caption", DIR, None,
    Some(Rc::new(complex_tests_dispatch)),
  );
}

#[test]
fn hyperchars_test() {
  latexml_test_single(
    "tests/complex/hyperchars.tex", "hyperchars", DIR, None,
    Some(Rc::new(complex_tests_dispatch)),
  );
}
