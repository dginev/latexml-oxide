///**********************************************************************
/// Test cases for latexml_oxide — graphics suite
///**********************************************************************
use latexml::util::test::*;
use phf::phf_map;
use std::rc::Rc;

const DIR: &str = "tests/graphics";
static REQUIRES: phf::Map<&'static str, &'static str> = phf_map! {
  "colors" => "dvipsnam.def",
  "xcolors" => "dvipsnam.def"
};

#[test]
fn calc_test() {
  latexml_test_single("tests/graphics/calc.tex", "calc", DIR, None, None);
}

#[test]
fn colors_test() {
  latexml_test_single("tests/graphics/colors.tex", "colors", DIR, Some(&REQUIRES), None);
}

#[test]
fn framed_test() {
  latexml_test_single("tests/graphics/framed.tex", "framed", DIR, None, None);
}

#[test]
fn graphrot_test() {
  latexml_test_single("tests/graphics/graphrot.tex", "graphrot", DIR, None, None);
}

#[test]
fn keyval_test() {
  latexml_test_single("tests/graphics/keyval.tex", "keyval", DIR, None,
    Some(Rc::new(latexml_contrib::dispatch)));
}

#[test]

fn picture_test() {
  latexml_test_single("tests/graphics/picture.tex", "picture", DIR, None, None);
}

#[test]
fn simplekv_test() {
  latexml_test_single("tests/graphics/simplekv.tex", "simplekv", DIR, None, None);
}

#[test]
fn xcolors_test() {
  latexml_test_single("tests/graphics/xcolors.tex", "xcolors", DIR, Some(&REQUIRES), None);
}

#[test]
#[ignore] // needs xy.sty
fn xytest_test() {
  latexml_test_single("tests/graphics/xytest.tex", "xytest", DIR, None, None);
}
