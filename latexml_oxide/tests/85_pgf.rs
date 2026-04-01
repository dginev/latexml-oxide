use latexml::util::test::*;
const DIR: &str = "tests/pgf";

#[test]
fn stress_pgfmath_test() {
  latexml_test_single("tests/pgf/stress_pgfmath.tex", "stress_pgfmath", DIR, None, None);
}

#[test]
fn stress_pgfplots_test() {
  latexml_test_single("tests/pgf/stress_pgfplots.tex", "stress_pgfplots", DIR, None, None);
}
