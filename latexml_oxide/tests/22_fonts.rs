// Font tests — one #[test] fn per `tests/fonts/*.tex+.xml` pair,
// generated at compile time by `tex_tests!`. Drop a new pair into
// `tests/fonts/` and it is picked up automatically on `cargo clean`
// + rebuild.
use latexml::tex_tests;

tex_tests!("tests/fonts");
