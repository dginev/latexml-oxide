// Keyval tests — one #[test] fn per `tests/keyval/*.tex+.xml` pair,
// generated at compile time by `tex_tests!`.
//
// Three tests (`keyvalstyle`, `xkeyvalstyle`, `xkeyvalview`) need the
// `latexml_contrib` dispatcher to find test-local .sty files via the
// noltxml raw-TeX passthrough. The remaining five don't, but
// `latexml_contrib::dispatch` is a strict no-op for files it doesn't
// recognise (returns `None`, the engine falls through to default
// binding lookup), so applying it directory-wide is safe and keeps
// the refactor clean.
use std::rc::Rc;

use latexml::tex_tests;

tex_tests!(
  "tests/keyval",
  None,
  Some(Rc::new(latexml_contrib::dispatch))
);
