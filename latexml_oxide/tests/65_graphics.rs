///**********************************************************************
/// Test cases for latexml_oxide — graphics suite
///**********************************************************************
//
// One #[test] fn per `tests/graphics/*.tex+.xml` pair, generated at
// compile time by `tex_tests!`.
//
// (The former `keyval` test here was a byte-identical duplicate of
// `keyval_rhai/keyvalstyle`, which now exercises the same conversion via a local
// `mykeyval.sty.rhai` fixture; it was removed rather than duplicated into a
// second gated dir.) `latexml_contrib::dispatch` is still passed directory-wide
// — a strict no-op for files it doesn't recognise — to stay consistent with the
// other suites.
//
use std::rc::Rc;

use latexml::{
  tex_tests,
  util::test::{RequirementTable, Requirements},
};

// Perl t/65_graphics.t.
static REQUIRES: &RequirementTable = &[
  ("colors", Requirements::packages(&["dvipsnam.def"])),
  ("xcolors", Requirements::packages(&["dvipsnam.def"])),
  ("xytest", Requirements::packages(&["xy.sty"])),
];

tex_tests!(
  "tests/graphics",
  Some(REQUIRES),
  Some(Rc::new(latexml_contrib::dispatch))
);
