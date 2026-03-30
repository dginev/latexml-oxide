use latexml::util::test::*;
const DIR: &str = "tests/tikz";

#[test]
fn tikz_3d_cone_test() {
  latexml_test_single("tests/tikz/3d-cone.tex", "3d-cone", DIR, None, None);
}

#[test]
fn ac_drive_components_test() {
  latexml_test_single("tests/tikz/ac-drive-components.tex", "ac-drive-components", DIR, None, None);
}

#[test]
fn ac_drive_voltage_test() {
  latexml_test_single("tests/tikz/ac-drive-voltage.tex", "ac-drive-voltage", DIR, None, None);
}

#[test]
fn atoms_and_orbitals_test() {
  latexml_test_single("tests/tikz/atoms-and-orbitals.tex", "atoms-and-orbitals", DIR, None, None);
}

#[test]
#[ignore] // >1000 errors from unported tikz shapes/arrows libraries
fn consort_flowchart_test() {
  latexml_test_single("tests/tikz/consort-flowchart.tex", "consort-flowchart", DIR, None, None);
}

#[test]
fn cycle_test() {
  latexml_test_single("tests/tikz/cycle.tex", "cycle", DIR, None, None);
}

#[test]
#[ignore] // exceeds 30M token limit — massive pgf math computations
fn dominoes_test() {
  latexml_test_single("tests/tikz/dominoes.tex", "dominoes", DIR, None, None);
}

#[test]
fn tikz_figure_test() {
  latexml_test_single("tests/tikz/tikz_figure.tex", "tikz_figure", DIR, None, None);
}

#[test]
#[ignore] // exceeds 30M token limit — massive pgf computations
fn unit_tests_by_silviu_test() {
  latexml_test_single("tests/tikz/unit_tests_by_silviu.tex", "unit_tests_by_silviu", DIR, None, None);
}

#[test]
fn various_colors_test() {
  latexml_test_single("tests/tikz/various_colors.tex", "various_colors", DIR, None, None);
}
