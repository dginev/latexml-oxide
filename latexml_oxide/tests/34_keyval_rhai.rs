// Keyval fixture tests redone with local `.rhai` bindings — faithful ports of
// Perl `t/keyval/{mykeyval,myxkeyval}.sty.ltxml`.
//
// `keyvalstyle` (mykeyval) and `xkeyvalstyle` (myxkeyval) were split out of
// `tests/keyval` so they can be gated to the `runtime-bindings` feature: each
// resolves its package from a local `<name>.sty.rhai` next to the `.tex`,
// discovered through the shared binding-resolution chain via the source
// directory. mykeyval/myxkeyval `RequirePackage` the real `keyval`/`xkeyval`
// (resolved by `latexml_package`), so no extra dispatcher is needed here.
#![cfg(feature = "runtime-bindings")]

use latexml::tex_tests;

tex_tests!("tests/keyval_rhai");
