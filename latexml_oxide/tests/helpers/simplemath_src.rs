/// Source-level bindings for simplemath.tex
/// Equivalent of Perl's simplemath.latexml
///
/// Declares math token roles for the simplemath test:
/// - f → FUNCTION (function application: f(x) → f@(x))
/// - a, b, x, D → ID
use latexml_package::prelude::*;

#[allow(dead_code)]
fn add_math_rewrite(match_char: &str, role: &str) -> Result<()> {
  let xpath = format!(
    "descendant-or-self::ltx:XMTok[text()='{}' and not(@meaning)][@_pvis and @_cvis]",
    match_char
  );
  let options = latexml_core::rewrite::RewriteOptions {
    xpath: Some(xpath),
    attributes: Some(format!("role='{role}'")),
    is_math: true,
    select_count: Some(1),
    ..Default::default()
  };
  state::push_value(
    "DOCUMENT_REWRITE_RULES",
    latexml_core::rewrite::Rewrite::new("math", options),
  )
}

#[rustfmt::skip]
LoadDefinitions!({
  // Enable speculative function application for f(x) patterns
  AssignValue!("MATHPARSER_SPECULATE" => true, Scope::Global);

  // Global role assignments (matching simplemath.latexml)
  add_math_rewrite("a", "ID")?;
  add_math_rewrite("b", "ID")?;
  add_math_rewrite("x", "ID")?;
  add_math_rewrite("D", "ID")?;
  add_math_rewrite("f", "FUNCTION")?;

  // Scoped rewrites: within label:sec:restricted, a and b become FUNCTION
  // Perl: DefMathRewrite(scope => 'label:sec:restricted', match => 'a', attributes => { role => 'FUNCTION' });
  // TODO: implement scope support for per-section role overrides
});
