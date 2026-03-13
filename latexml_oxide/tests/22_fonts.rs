// Font tests — no infinite loops in test harness (binary hangs are separate).
// Tests need cargo clean after adding new .tex/.xml pairs.
//
// Blocked tests (0 diffs but crash):
//   acc, cancels, soul, mathaccents, abxtest, mathcolor, stmaryrd, wasysym
//     → need \DeclareMathAccent, \DeclareMathSymbol, \Gin, \ExplSyntaxOn, \newfont
//   omencodings (1 diff) → known OML font map single-char limitation (U+0311 vs U+0361)
//   sizes (crash) → many diffs after lastkern fix
use latexml::util::test::*;
const DIR: &str = "tests/fonts";

#[test]
fn textsymbols_test() {
  latexml_test_single("tests/fonts/textsymbols.tex", "textsymbols", DIR, None, None);
}

#[test]
fn emph_test() {
  latexml_test_single("tests/fonts/emph.tex", "emph", DIR, None, None);
}
