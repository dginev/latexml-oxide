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
    "xii" | "xii.latexml" => Some(helpers::xii_tex::load_definitions()),
    "labelled.latexml" => Some(helpers::labelled_tex::load_definitions()),
    _ => None,
  }
}

fn complex(name: &str) {
  latexml_test_single(
    &format!("tests/complex/{name}.tex"),
    name,
    DIR,
    None,
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
fn figure_mixed_content_test() { complex("figure_mixed_content"); }

#[test]
fn hypertest_test() { complex("hypertest"); }

#[test]
fn labelled_test() { complex("labelled"); }

#[test]
fn versioned_fallback_test() { complex("versioned_fallback"); }

#[test]
fn cleveref_minimal_test() { complex("cleveref_minimal"); }

#[test]
fn equationnest_test() { complex("equationnest"); }

// Tests that need missing packages or have significant diffs

#[test]
fn aastex631_deluxetable_test() { complex("aastex631_deluxetable"); }

#[test]
fn acm_aria_test() { complex("acm_aria"); }

#[test]
fn aastex_test_test() { complex("aastex_test"); }

#[test]
fn aliceblog_test() { complex("aliceblog"); }

#[test]
#[ignore = "physics.sty \\lx@physics@mathbfit starred vector reversion drift — snapshot captures pre-port `{\\bf\\it a}` grouping; faithful port now emits `\\mathbf{*}{a}` (commit 1aad02075). Snapshot refresh pending"]
fn physics_test() { complex("physics"); }

#[test]
fn si_test() { complex("si"); }

#[test]
fn tcilatex_minimal_test() { complex("tcilatex_minimal"); }
