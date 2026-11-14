use latexml::{
  tex_tests,
  util::test::{RequirementTable, Requirements},
};

// Perl t/83_expl3.t, without its `env => 'CI'`: the local gates run these fixtures too.
static REQUIRES: &RequirementTable = &[
  (
    "tilde_tricks",
    Requirements::packages(&["expl3.sty"]).texlive_min(2018),
  ),
  (
    "xparse",
    Requirements::packages(&["expl3.sty", "xparse.sty"]).texlive_min(2019),
  ),
];

tex_tests!("tests/expl3", Some(REQUIRES), None);
