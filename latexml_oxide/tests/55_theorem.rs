///**********************************************************************
/// Test cases for latexml_oxide — theorem tests
///**********************************************************************
use latexml::tex_tests;
use latexml::util::test::{RequirementTable, Requirements};

// Perl t/55_theorem.t (ntheoremstyle reads the same file).
static REQUIRES: &RequirementTable = &[
  ("ntheorem", Requirements::packages(&["ntheorem.std"])),
  ("ntheoremstyle", Requirements::packages(&["ntheorem.std"])),
];

tex_tests!("tests/theorem", Some(REQUIRES), None);
