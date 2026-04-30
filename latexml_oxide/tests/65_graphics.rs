///**********************************************************************
/// Test cases for latexml_oxide — graphics suite
///**********************************************************************
//
// One #[test] fn per `tests/graphics/*.tex+.xml` pair, generated at
// compile time by `tex_tests!`.
//
// One test (`keyval`) needs the `latexml_contrib` dispatcher to find
// test-local .sty files via the noltxml raw-TeX passthrough. The
// other 8 don't, but `latexml_contrib::dispatch` is a strict no-op
// for files it doesn't recognise, so applying it directory-wide is
// safe.
//
// A previous hand-written version registered a REQUIRES map gating
// `colors` / `xcolors` on the presence of `dvipsnam.def`. The
// `validate_requirements` runtime helper is currently a TODO stub
// returning `true` unconditionally (see `util/test.rs:100`), so the
// map was cosmetic. If REQUIRES gating is ever implemented, it will
// need to be reintroduced here — probably as an `ignored_if_missing!`
// macro invoked from inside the generated tests, not as a static
// directory-level attribute.
use std::rc::Rc;

use latexml::tex_tests;

tex_tests!(
  "tests/graphics",
  None,
  Some(Rc::new(latexml_contrib::dispatch))
);
