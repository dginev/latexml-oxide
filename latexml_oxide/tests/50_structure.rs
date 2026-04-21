// Structure tests — one #[test] fn per `tests/structure/*.tex+.xml`
// pair, generated at compile time by `tex_tests!`.
//
// Two tests (`filelist`, `options`) need the `latexml_contrib` dispatcher
// to find their test-local .cls/.sty files
// (`filelistclass.cls`, `myclass.cls`, `apackage.sty`) via the
// noltxml raw-TeX passthrough. The other 40 tests don't need it.
// `latexml_contrib::dispatch` returns `None` for files it doesn't
// recognise, so the engine falls through to its normal binding
// lookup — no observable change for the 40 non-contrib tests.
// Applying it directory-wide is therefore safe and keeps the
// refactor clean.
use std::rc::Rc;

use latexml::tex_tests;

tex_tests!(
  "tests/structure",
  None,
  Some(Rc::new(latexml_contrib::dispatch))
);
