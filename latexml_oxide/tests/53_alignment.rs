// Alignment (tabular / array / align / gather / multline) tests —
// one #[test] fn per `tests/alignment/*.tex+.xml` pair, generated at
// compile time by `tex_tests!`. Note: an earlier comment here warned
// that the aggregated form of tex_tests leaked memory across runs;
// that concern does not apply to the per-test form (each generated
// #[test] fn runs in its own cargo-test thread with isolated State).
use latexml::{
  tex_tests,
  util::test::{RequirementTable, Requirements},
};

// Perl t/53_alignment.t.
static REQUIRES: &RequirementTable = &[
  ("listing", Requirements::packages(&["listings.cfg"])),
  (
    "algx",
    Requirements::packages(&["algorithm.sty", "algpseudocode.sty"]),
  ),
  ("cells", Requirements::packages(&["makecell.sty"])),
];

tex_tests!("tests/alignment", Some(REQUIRES), None);
