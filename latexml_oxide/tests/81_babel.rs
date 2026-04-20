// Babel tests — one #[test] fn per `tests/babel/*.tex+.xml` pair,
// generated at compile time by `tex_tests!`. Drop a new pair into
// `tests/babel/` and it is picked up automatically on `cargo clean`
// + rebuild. See `latexml_codegen::testable::compile_tests_at`.
use latexml::tex_tests;

tex_tests!("tests/babel");
