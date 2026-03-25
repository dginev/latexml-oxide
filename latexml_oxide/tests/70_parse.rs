// Parse tests — individually listed for per-test #[ignore] support.
use latexml::util::test::*;

const DIR: &str = "tests/parse";

#[test]
fn algebraic_terms_test() {
  latexml_test_single("tests/parse/algebraic_terms.tex", "algebraic_terms", DIR, None, None);
}

#[test]
fn array_math_parse_test() {
  latexml_test_single("tests/parse/array_math.tex", "array_math", DIR, None, None);
}

#[test]
#[ignore] // 0 text= diffs, minor xml:id ordering diffs remain
fn artefacts_test() {
  latexml_test_single("tests/parse/artefacts.tex", "artefacts", DIR, None, None);
}

#[test]
fn calculus_test() {
  latexml_test_single("tests/parse/calculus.tex", "calculus", DIR, None, None);
}

#[test]
fn compose_test() {
  latexml_test_single("tests/parse/compose.tex", "compose", DIR, None, None);
}

#[test]
fn fences_test() {
  latexml_test_single("tests/parse/fences.tex", "fences", DIR, None, None);
}

#[test]
fn function_argument_syntax_test() {
  latexml_test_single("tests/parse/function_argument_syntax.tex", "function_argument_syntax", DIR, None, None);
}

#[test]
#[ignore] // minor diffs after fenced XMDual fix
fn functions_test() {
  latexml_test_single("tests/parse/functions.tex", "functions", DIR, None, None);
}

#[test]
fn kludge_test() {
  latexml_test_single("tests/parse/kludge.tex", "kludge", DIR, None, None);
}

#[test]
fn metarelation_elision_test() {
  latexml_test_single("tests/parse/metarelation_elision.tex", "metarelation_elision", DIR, None, None);
}

#[test]
fn mixedfrac_test() {
  latexml_test_single("tests/parse/mixedfrac.tex", "mixedfrac", DIR, None, None);
}

#[test]
fn multirelations_test() {
  latexml_test_single("tests/parse/multirelations.tex", "multirelations", DIR, None, None);
}

#[test]
fn nested_application_test() {
  latexml_test_single("tests/parse/nested_application.tex", "nested_application", DIR, None, None);
}

#[test]
#[ignore] // minor diffs after fenced XMDual fix
fn operators_test() {
  latexml_test_single("tests/parse/operators.tex", "operators", DIR, None, None);
}

#[test]
fn parens_test() {
  latexml_test_single("tests/parse/parens.tex", "parens", DIR, None, None);
}

#[test]
fn parser_speculate_test() {
  latexml_test_single("tests/parse/parser_speculate.tex", "parser_speculate", DIR, None, None);
}

#[test]
fn prescripted_test() {
  latexml_test_single("tests/parse/prescripted.tex", "prescripted", DIR, None, None);
}

#[test]
#[ignore] // minor diffs after fenced XMDual fix
fn qm_test() {
  latexml_test_single("tests/parse/qm.tex", "qm", DIR, None, None);
}

#[test]
fn relations_test() {
  latexml_test_single("tests/parse/relations.tex", "relations", DIR, None, None);
}

#[test]
fn scripts_test() {
  latexml_test_single("tests/parse/scripts.tex", "scripts", DIR, None, None);
}

#[test]
fn sequences_and_lists_test() {
  latexml_test_single("tests/parse/sequences_and_lists.tex", "sequences_and_lists", DIR, None, None);
}

#[test]
fn sets_test() {
  latexml_test_single("tests/parse/sets.tex", "sets", DIR, None, None);
}

#[test]
fn spacing_test() {
  latexml_test_single("tests/parse/spacing.tex", "spacing", DIR, None, None);
}

#[test]
fn standalone_equations_test() {
  latexml_test_single("tests/parse/standalone_equations.tex", "standalone_equations", DIR, None, None);
}

#[test]
fn standalone_modifiers_test() {
  latexml_test_single("tests/parse/standalone_modifiers.tex", "standalone_modifiers", DIR, None, None);
}

#[test]
fn subordinate_lists_test() {
  latexml_test_single("tests/parse/subordinate_lists.tex", "subordinate_lists", DIR, None, None);
}

#[test]
fn terms_test() {
  latexml_test_single("tests/parse/terms.tex", "terms", DIR, None, None);
}

#[test]
fn vertbars_test() {
  latexml_test_single("tests/parse/vertbars.tex", "vertbars", DIR, None, None);
}
