/// Source-level bindings for simplemath.tex
/// Equivalent of Perl's simplemath.latexml
///
/// Declares math token roles for the simplemath test:
/// - f → FUNCTION (function application: f(x) → f@(x))
/// - \hat{f} → ID (accented f is multiplicative, not a function)
/// - f_D → DIFFOP (differential operator)
/// - f_* → ID (any other subscripted f is an identifier)
/// - a, b, x, D → ID
use latexml_package::prelude::*;

#[allow(dead_code)]
fn add_math_rewrite(match_char: &str, role: &str) -> Result<()> {
  add_math_rewrite_scoped(match_char, role, None)
}

/// Add scoped rewrite FIRST (prepend) — matches Perl's UnshiftValue for \lxDeclare
#[allow(dead_code)]
fn add_math_rewrite_scoped_first(match_char: &str, role: &str, scope: &str) -> Result<()> {
  let xpath = format!(
    "descendant-or-self::ltx:XMTok[text()='{}' and not(@meaning)][@_pvis and @_cvis]",
    match_char
  );
  let mut attrs_map = rustc_hash::FxHashMap::default();
  attrs_map.insert("role".to_string(), role.to_string());
  let options = RewriteOptions {
    xpath: Some(xpath),
    scope: Some(Scope::Named(pin(scope))),
    attributes_map: Some(attrs_map),
    is_math: true,
    select_count: Some(1),
    ..Default::default()
  };
  unshift_value("DOCUMENT_REWRITE_RULES", vec![Rewrite::new(
    "math", options,
  )]);
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
  let options = RewriteOptions {
    xpath: Some(xpath),
    scope: scope.map(|s| Scope::Named(pin(s))),
    attributes_map: Some(attrs_map),
    is_math: true,
    select_count: Some(1),
    ..Default::default()
  };
  push_value("DOCUMENT_REWRITE_RULES", Rewrite::new("math", options))
}

/// \hat{f} → ID: accented f treated as identifier, not function.
/// Single-node match (nnodes=1): the XMApp containing hat accent + f.
#[allow(dead_code)]
fn add_accent_f_rewrite() -> Result<()> {
  // Use * instead of ltx:XMTok in nested predicates to avoid
  // namespace resolution issues in libxml2 XPath evaluation.
  let xpath = "descendant-or-self::ltx:XMApp\
    [*[@name='hat' and @role='OVERACCENT']]\
    [*[text()='f'] or */*[text()='f']]"
    .to_string();
  let mut attrs_map = rustc_hash::FxHashMap::default();
  attrs_map.insert("role".to_string(), "ID".to_string());
  let options = RewriteOptions {
    xpath: Some(xpath),
    attributes_map: Some(attrs_map),
    is_math: true,
    select_count: Some(1),
    ..Default::default()
  };
  push_value("DOCUMENT_REWRITE_RULES", Rewrite::new("math", options))
}

/// f_D → DIFFOP: f subscripted by D is a differential operator.
/// Two-sibling match (nnodes=2): XMTok[f] + XMApp[POSTSUBSCRIPT[D]].
/// Wrapped in XMWrap[role=DIFFOP] by the rewrite engine.
#[allow(dead_code)]
fn add_f_d_diffop_rewrite() -> Result<()> {
  let xpath = "descendant-or-self::ltx:XMTok[text()='f' and not(@meaning)]\
    [following-sibling::*[1]\
    [self::ltx:XMApp[@role='POSTSUBSCRIPT' and normalize-space(.)='D']]]\
    [@_pvis and @_cvis]"
    .to_string();
  let mut attrs_map = rustc_hash::FxHashMap::default();
  attrs_map.insert("role".to_string(), "DIFFOP".to_string());
  let options = RewriteOptions {
    xpath: Some(xpath),
    attributes_map: Some(attrs_map),
    is_math: true,
    select_count: Some(2),
    ..Default::default()
  };
  push_value("DOCUMENT_REWRITE_RULES", Rewrite::new("math", options))
}

/// f_* → ID: f with any subscript (except D, handled above) is an identifier.
/// Two-sibling match (nnodes=2): XMTok[f] + XMApp[POSTSUBSCRIPT].
/// Wrapped in XMDual[role=ID] by the rewrite engine (via wildcard_paths).
/// The f_D rule fires first and marks those nodes, so this only catches
/// remaining subscripted-f patterns (f_1, f_2, etc.).
#[allow(dead_code)]
fn add_f_wildcard_rewrite() -> Result<()> {
  let xpath = "descendant-or-self::ltx:XMTok[text()='f' and not(@meaning)]\
    [following-sibling::*[1][self::ltx:XMApp[@role='POSTSUBSCRIPT']]]\
    [@_pvis and @_cvis]"
    .to_string();
  let mut attrs_map = rustc_hash::FxHashMap::default();
  attrs_map.insert("role".to_string(), "ID".to_string());
  let options = RewriteOptions {
    xpath: Some(xpath),
    attributes_map: Some(attrs_map),
    // Wildcard = child 1 of sibling 2 (subscript content in POSTSUBSCRIPT XMApp)
    wildcard_paths: Some(vec![vec![2, 1]]),
    is_math: true,
    select_count: Some(2),
    ..Default::default()
  };
  push_value("DOCUMENT_REWRITE_RULES", Rewrite::new("math", options))
}

#[rustfmt::skip]
LoadDefinitions!({
  // Enable speculative function application for f(x) patterns.
  // The actual rewrite rules (role assignments, wildcard patterns, etc.) are loaded
  // from simplemath.latexml by the .latexml file loader in core_interface.rs.
  AssignValue!("MATHPARSER_SPECULATE" => true, Scope::Global);
});
