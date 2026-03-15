// Keyval options tests -- depend on xkeyval package and test-local .sty packages.
// Test-local packages are loaded via latexml_contrib dispatcher (noltxml passthrough).
use std::rc::Rc;

use latexml::util::test::*;
use latexml_core::common::error::Result;
use latexml_core::state;
const DIR: &str = "tests/keyval_options";

pub fn keyval_options_dispatch(filename: &str) -> Option<Result<()>> {
  // Enable raw TeX loading so test-local .sty/.cls files are found
  state::assign_value("INCLUDE_STYLES", true, None);
  latexml_contrib::dispatch(filename)
}

#[test]
fn xkvdop1a_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop1a.tex",
    "xkvdop1a",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}

#[test]
fn xkvdop1b_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop1b.tex",
    "xkvdop1b",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}

#[test]
fn xkvdop2a_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop2a.tex",
    "xkvdop2a",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}

#[test]
fn xkvdop2b_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop2b.tex",
    "xkvdop2b",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}

#[test]
fn xkvdop3a_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop3a.tex",
    "xkvdop3a",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}

#[test]
fn xkvdop3b_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop3b.tex",
    "xkvdop3b",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}

#[test]
fn xkvdop4a_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop4a.tex",
    "xkvdop4a",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}

#[test]
fn xkvdop5a_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop5a.tex",
    "xkvdop5a",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}

#[test]
fn xkvdop5b_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop5b.tex",
    "xkvdop5b",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}

#[test]
fn xkvdop6a_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop6a.tex",
    "xkvdop6a",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}

#[test]
fn xkvdop6b_test() {
  latexml_test_single(
    "tests/keyval_options/xkvdop6b.tex",
    "xkvdop6b",
    DIR,
    None,
    Some(Rc::new(keyval_options_dispatch)),
  );
}
