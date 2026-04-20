// Keyval options tests -- depend on xkeyval package and test-local .sty packages.
// Test-local packages are loaded via latexml_contrib dispatcher (noltxml passthrough).
use std::rc::Rc;

use latexml::tex_tests;
use latexml_core::common::error::Result;
use latexml_core::state;

pub fn keyval_options_dispatch(filename: &str) -> Option<Result<()>> {
  // Enable raw TeX loading so test-local .sty/.cls files are found
  state::assign_value("INCLUDE_STYLES", true, None);
  latexml_contrib::dispatch(filename)
}

tex_tests!(
  "tests/keyval_options",
  None,
  Some(Rc::new(keyval_options_dispatch))
);
