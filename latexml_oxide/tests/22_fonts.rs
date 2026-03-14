// Font tests — no infinite loops in test harness (binary hangs are separate).
// Tests need cargo clean after adding new .tex/.xml pairs.
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

// -- Tests with diffs (need targeted fixes) --

#[test]
fn accents_test() {
  latexml_test_single("tests/fonts/accents.tex", "accents", DIR, None, None);
}

#[test]
#[ignore] // diffs — \fontname not implemented (shows "fontname not implemented" instead of cmr10)
fn fonts_test() {
  latexml_test_single("tests/fonts/fonts.tex", "fonts", DIR, None, None);
}

#[test]
#[ignore] // diffs — math parser (XMDual/XMApp structure)
fn mixed_test() {
  latexml_test_single("tests/fonts/mixed.tex", "mixed", DIR, None, None);
}

#[test]
#[ignore] // diffs — \fontname not implemented (shows "fontname not implemented" instead of cmr10)
fn plainfonts_test() {
  latexml_test_single("tests/fonts/plainfonts.tex", "plainfonts", DIR, None, None);
}

#[test]
fn textcomp_test() {
  latexml_test_single("tests/fonts/textcomp.tex", "textcomp", DIR, None, None);
}

#[test]
fn ulem_test() {
  latexml_test_single("tests/fonts/ulem.tex", "ulem", DIR, None, None);
}

#[test]
fn omencodings_test() {
  latexml_test_single("tests/fonts/omencodings.tex", "omencodings", DIR, None, None);
}

#[test]
#[ignore] // diffs — math parser
fn mathbbol_test() {
  latexml_test_single("tests/fonts/mathbbol.tex", "mathbbol", DIR, None, None);
}

#[test]
fn bbold_test() {
  latexml_test_single("tests/fonts/bbold.tex", "bbold", DIR, None, None);
}

#[test]
#[ignore] // diffs — needs pifont package (pzd font map)
fn ding_test() {
  latexml_test_single("tests/fonts/ding.tex", "ding", DIR, None, None);
}

#[test]
#[ignore] // crash in math parser — todo!() not implemented
fn esint_test() {
  latexml_test_single("tests/fonts/esint.tex", "esint", DIR, None, None);
}

#[test]
fn marvosym_test() {
  latexml_test_single("tests/fonts/marvosym.tex", "marvosym", DIR, None, None);
}

// -- Tests that crash (need package/subsystem work) --

#[test]
#[ignore] // diffs — math parser (XMDual structure, text= attribute, list refs)
fn acc_test() {
  latexml_test_single("tests/fonts/acc.tex", "acc", DIR, None, None);
}

#[test]
#[ignore] // diffs — tabular border="r", \underbrace section structure
fn mathaccents_test() {
  latexml_test_single("tests/fonts/mathaccents.tex", "mathaccents", DIR, None, None);
}

#[test]
#[ignore] // needs stmaryrd.sty symbols via \DeclareMathSymbol
fn stmaryrd_test() {
  latexml_test_single("tests/fonts/stmaryrd.tex", "stmaryrd", DIR, None, None);
}

#[test]
fn mathcolor_test() {
  latexml_test_single("tests/fonts/mathcolor.tex", "mathcolor", DIR, None, None);
}

#[test]
#[ignore] // needs \Gin, \ExplSyntaxOn (graphics/expl3)
fn wasysym_test() {
  latexml_test_single("tests/fonts/wasysym.tex", "wasysym", DIR, None, None);
}

#[test]
fn cancels_test() {
  latexml_test_single("tests/fonts/cancels.tex", "cancels", DIR, None, None);
}

#[test]
fn soul_test() {
  latexml_test_single("tests/fonts/soul.tex", "soul", DIR, None, None);
}

#[test]
#[ignore] // needs \hexnumber@, \mathxfam (font allocation)
fn abxtest_test() {
  latexml_test_single("tests/fonts/abxtest.tex", "abxtest", DIR, None, None);
}

#[test]
#[ignore] // many diffs after lastkern fix
fn sizes_test() {
  latexml_test_single("tests/fonts/sizes.tex", "sizes", DIR, None, None);
}
