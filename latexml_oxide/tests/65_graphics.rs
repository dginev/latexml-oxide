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
