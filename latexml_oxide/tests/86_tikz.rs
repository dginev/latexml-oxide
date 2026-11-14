use latexml::{
  tex_tests,
  util::test::{RequirementTable, Requirements},
};

// Goldens of newer packages whose output really changed (pdflatex agrees): pgf r79866's blank line closing
// `\pgf@setup@shading@model` (pgfcoreshade.code.tex:378) opens a paragraph after `\pgfdeclareverticalshading`;
// circuitikz 1.8.6's transistor gate code (`transistors/gate thickness`, pgfcirctripoles.tex) draws longer IGBT bars.
static REQUIRES: &RequirementTable = &[
  (
    "unit_tests_by_silviu",
    Requirements::packages(&[]).golden_variant("pgf-r79866", "pgfcoreshade.code.tex", |text| {
      text
        .split("\\def\\pgf@setup@shading@model{")
        .nth(1)
        .and_then(|body| body.split("\n}").next())
        .is_some_and(|body| body.split('\n').any(|line| line.trim().is_empty()))
    }),
  ),
  (
    "ac-drive-components",
    Requirements::packages(&[]).golden_variant("circuitikz-1.8.6", "pgfcirctripoles.tex", |text| {
      text.contains("transistors/gate thickness")
    }),
  ),
];

tex_tests!("tests/tikz", Some(REQUIRES), None);
