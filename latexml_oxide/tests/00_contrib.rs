///**********************************************************************
/// Test cases for latexml_oxide — contributed (`latexml_contrib`) bindings
///**********************************************************************
//
// One #[test] fn per `tests/contrib/*.tex+.xml` pair, generated at compile time
// by `tex_tests!` — the same shape as the other suites (`65_graphics.rs`, …).
//
// This used to call the RUNTIME `latexml_tests_internal("tests/contrib", …)`,
// which globs and converts every pair inside a SINGLE #[test] fn, i.e. a single
// libtest thread. That is fine for one fixture and aborts on two: the engine's
// `#[thread_local]` state is built for one conversion per thread, so the second
// `initialize_singletons` in the same thread re-runs `Let!` against stale
// interner ids and trips a `slice::get_unchecked` precondition (SIGABRT, not a
// catchable panic). `tex_tests!` gives every pair its own #[test] — libtest runs
// each on a fresh thread — which is why every other directory-wide suite in the
// tree already uses it.
use std::rc::Rc;

use latexml::tex_tests;

tex_tests!(
  "tests/contrib",
  None,
  Some(Rc::new(latexml_contrib::dispatch))
);
