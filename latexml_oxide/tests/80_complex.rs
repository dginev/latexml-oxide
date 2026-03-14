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

fn complex(name: &str) {
  latexml_test_single(
    &format!("tests/complex/{name}.tex"),
    name, DIR, None,
    Some(Rc::new(complex_tests_dispatch)),
  );
}

#[test]
fn xii_test() { complex("xii"); }

#[test]
fn figure_dual_caption_test() { complex("figure_dual_caption"); }

#[test]
fn hyperchars_test() { complex("hyperchars"); }

#[test]
#[ignore] // crash — \lstKV@SetIf@ parameter spec error (listings.sty)
fn figure_mixed_content_test() { complex("figure_mixed_content"); }

#[test]
#[ignore] // diffs — hypersetup metadata, colorlinks color wrapping
fn hypertest_test() { complex("hypertest"); }

#[test]
#[ignore] // diffs — label-based numbering
fn labelled_test() { complex("labelled"); }

#[test]
fn versioned_fallback_test() { complex("versioned_fallback"); }

#[test]
#[ignore] // diffs — cleveref references
fn cleveref_minimal_test() { complex("cleveref_minimal"); }

#[test]
fn equationnest_test() { complex("equationnest"); }

// Tests that need missing packages or have significant diffs

#[test]
#[ignore] // needs aastex631.cls binding
fn aastex631_deluxetable_test() { complex("aastex631_deluxetable"); }

#[test]
#[ignore] // needs acmart.cls binding
fn acm_aria_test() { complex("acm_aria"); }

#[test]
#[ignore] // needs aastex.cls binding
fn aastex_test_test() { complex("aastex_test"); }

#[test]
#[ignore] // needs blog.cls binding
fn aliceblog_test() { complex("aliceblog"); }

#[test]
#[ignore] // diffs — physics package
fn physics_test() { complex("physics"); }

#[test]
#[ignore] // diffs — siunitx package
fn si_test() { complex("si"); }

#[test]
#[ignore] // needs tcilatex package
fn tcilatex_minimal_test() { complex("tcilatex_minimal"); }
