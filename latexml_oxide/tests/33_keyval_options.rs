// Keyval-options tests. The `xkvdop{1-6}` packages are test-local fixtures: a
// local `xkvdop*.{sty,cls}.rhai` next to each `.tex` raw-loads the bundled
// `.sty`/`.cls` (faithful to the Perl `t/keyval_options/xkvdop*.ltxml`, which
// `InputDefinitions(... noltxml => 1)`). Discovery rides the shared
// binding-resolution chain (rhai > contrib > package) via the source-dir search
// path — so this whole group requires the `runtime-bindings` feature and is
// skipped when it is disabled. (`keysetopt` still resolves through the
// `latexml_contrib` dispatcher passed as tier 2.)
#![cfg(feature = "runtime-bindings")]

use std::rc::Rc;

use latexml::tex_tests;
use latexml_core::{common::error::Result, state};

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
