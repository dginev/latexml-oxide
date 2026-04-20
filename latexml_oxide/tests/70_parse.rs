// Parse tests — one #[test] fn per `tests/parse/*.tex+.xml` pair,
// generated at compile time by `tex_tests!`. Per-test #[ignore] is
// still possible on the expanded fns; add it directly to a specific
// test via an `attr.rs`-style `#[test_case_attrs]` plus the
// macro-side augmentation if needed, or simply rename the .tex to
// .tex.ignored to drop it from auto-discovery.
use latexml::tex_tests;

tex_tests!("tests/parse");
