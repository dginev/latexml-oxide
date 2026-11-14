// Babel tests — one #[test] fn per `tests/babel/*.tex+.xml` pair,
// generated at compile time by `tex_tests!`. Drop a new pair into
// `tests/babel/` and it is picked up automatically on `cargo clean`
// + rebuild. See `latexml_codegen::testable::compile_tests_at`.
use latexml::{
  tex_tests,
  util::test::{RequirementTable, Requirements},
};

// Perl t/81_babel.t.
static REQUIRES: &RequirementTable = &[
  ("*", Requirements::packages(&["babel.sty", "babel.def"])),
  (
    "csquotes",
    Requirements::packages(&["csquotes.sty", "frenchb.ldf", "germanb.ldf"]).texlive_min(2015),
  ),
  ("numprints", Requirements::packages(&["numprint.sty"])),
  ("german", Requirements::packages(&["germanb.ldf"])),
  (
    "greek",
    Requirements::packages(&["greek.ldf", "lgrenc.def"]),
  ),
  (
    "french",
    Requirements::packages(&["frenchb.ldf", "numprint.sty"]).texlive_min(2017),
  ),
  (
    "page545",
    Requirements::packages(&["germanb.ldf", "frenchb.ldf"]),
  ),
];

tex_tests!("tests/babel", Some(REQUIRES), None);
