// Structure fixture tests redone with local `.rhai` bindings — faithful ports
// of Perl `t/structure/{apackage.sty,filelistclass.cls,myclass.cls}.ltxml`.
//
// These two `.tex` (`filelist`, `options`) were split out of `tests/structure`
// so they can be gated to the `runtime-bindings` feature: each resolves its
// package/class from a local `<name>.<ext>.rhai` next to the `.tex`, discovered
// through the shared binding-resolution chain (rhai > contrib > package) via the
// source-directory search path — exactly as the Perl suite resolves a local
// `.ltxml`. `latexml_contrib::dispatch` is passed as tier 2 (a no-op for these
// fixtures; kept for any real package a future test here might pull in).
#![cfg(feature = "runtime-bindings")]

use std::rc::Rc;

use latexml::tex_tests;

tex_tests!(
  "tests/structure_rhai",
  None,
  Some(Rc::new(latexml_contrib::dispatch))
);
