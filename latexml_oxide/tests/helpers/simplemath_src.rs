/// Source-level bindings for simplemath.tex
/// Equivalent of Perl's simplemath.latexml
///
/// Declares math token roles for the simplemath test:
/// - f → FUNCTION (function application: f(x) → f@(x))
/// - a, b, x, D → ID
use latexml_package::prelude::*;

#[allow(dead_code)]
fn add_math_rewrite(match_char: &str, role: &str) -> Result<()> {
  add_math_rewrite_scoped(match_char, role, None)
}

#[allow(dead_code)]
/// Add scoped rewrite FIRST (prepend) — matches Perl's UnshiftValue for \lxDeclare
#[allow(dead_code)]
fn add_math_rewrite_scoped_first(match_char: &str, role: &str, scope: &str) -> Result<()> {
  let xpath = format!(
    "descendant-or-self::ltx:XMTok[text()='{}' and not(@meaning)][@_pvis and @_cvis]",
    match_char
  );
  let mut attrs_map = rustc_hash::FxHashMap::default();
  attrs_map.insert("role".to_string(), role.to_string());
  let options = latexml_core::rewrite::RewriteOptions {
    xpath: Some(xpath),
    scope: Some(latexml_core::state::Scope::Named(latexml_core::common::arena::pin(scope))),
    attributes_map: Some(attrs_map),
    is_math: true,
    select_count: Some(1),
    ..Default::default()
  };
  state::unshift_value(
    "DOCUMENT_REWRITE_RULES",
    vec![latexml_core::rewrite::Rewrite::new("math", options)],
  );
  Ok(())
}

#[allow(dead_code)]
fn add_math_rewrite_scoped(match_char: &str, role: &str, scope: Option<&str>) -> Result<()> {
  let xpath = format!(
    "descendant-or-self::ltx:XMTok[text()='{}' and not(@meaning)][@_pvis and @_cvis]",
    match_char
  );
  let mut attrs_map = rustc_hash::FxHashMap::default();
  attrs_map.insert("role".to_string(), role.to_string());
  // Perl scope strings: "label:sec:restricted" or "id:S1" — passed as Named(SymStr)
  let options = latexml_core::rewrite::RewriteOptions {
    xpath: Some(xpath),
    scope: scope.map(|s| latexml_core::state::Scope::Named(latexml_core::common::arena::pin(s))),
    attributes_map: Some(attrs_map),
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
  // MUST be added before global rewrites (UnshiftValue in Perl) so they take priority.
  add_math_rewrite_scoped_first("a", "FUNCTION", "label:sec:restricted")?;
  add_math_rewrite_scoped_first("b", "FUNCTION", "label:sec:restricted")?;

  // Perl: DefMathRewrite(match => '\hat{f}', attributes => { role => 'ID' });
  // \hat{f} treated as ID (multiplicative atom, not function)
  // TODO: multi-token match patterns like \hat{f}

  // Also add \hat{f} as ID (Perl simplemath.latexml)
  // DefMathRewrite(match => '\hat{f}', attributes => { role => 'ID' });
});
