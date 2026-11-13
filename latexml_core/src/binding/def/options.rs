//! The option keys each definer accepts — Perl `Package.pm`'s `$*_options` tables — and
//! `check_options`, Perl's `CheckOptions` (Package.pm:1062-1067). The runtime (Rhai) front-end
//! checks every option bag against these; the compile-time macros check theirs at expansion.

/// `DefMacro` (Package.pm:1136-1140, `$macro_options`).
pub const MACRO_OPTIONS: &[&str] = &[
  "scope",
  "locked",
  "mathactive",
  "protected",
  "robust",
  "outer",
  "long",
  "nopackParameters",
];

/// `DefConditional` (Package.pm:1185, `$conditional_options`).
pub const CONDITIONAL_OPTIONS: &[&str] = &["scope", "locked", "skipper"];

/// `DefPrimitive` (Package.pm:1277-1283, `$primitive_options`).
pub const PRIMITIVE_OPTIONS: &[&str] = &[
  "isPrefix",
  "scope",
  "font",
  "mode",
  "enterHorizontal",
  "leaveHorizontal",
  "requireMath",
  "forbidMath",
  "beforeDigest",
  "afterDigest",
  "bounded",
  "locked",
  "alias",
  "robust",
  "outer",
  "long",
];

/// `DefConstructor` (Package.pm:1432-1440, `$constructor_options`).
pub const CONSTRUCTOR_OPTIONS: &[&str] = &[
  "mode",
  "requireMath",
  "forbidMath",
  "font",
  "enterHorizontal",
  "leaveHorizontal",
  "alias",
  "reversion",
  "sizer",
  "properties",
  "nargs",
  "attributeForm",
  "beforeDigest",
  "afterDigest",
  "beforeConstruct",
  "afterConstruct",
  "captureBody",
  "scope",
  "bounded",
  "locked",
  "outer",
  "long",
  "robust",
];

/// `DefEnvironment` (Package.pm:1880-1888, `$environment_options`).
pub const ENVIRONMENT_OPTIONS: &[&str] = &[
  "mode",
  "requireMath",
  "forbidMath",
  "enterHorizontal",
  "leaveHorizontal",
  "properties",
  "nargs",
  "font",
  "beforeDigest",
  "afterDigest",
  "afterDigestBegin",
  "beforeDigestEnd",
  "afterDigestBody",
  "beforeConstruct",
  "afterConstruct",
  "reversion",
  "sizer",
  "scope",
  "locked",
  "attributeForm",
];

/// `DefMath` (Package.pm:1591-1601, `$math_options`).
pub const MATH_OPTIONS: &[&str] = &[
  "name",
  "meaning",
  "omcd",
  "reversion",
  "sizer",
  "alias",
  "role",
  "operator_role",
  "reorder",
  "dual",
  "decl_id",
  "mathstyle",
  "font",
  "scriptpos",
  "operator_scriptpos",
  "stretchy",
  "operator_stretchy",
  "lpadding",
  "rpadding",
  "beforeDigest",
  "afterDigest",
  "scope",
  "nogroup",
  "locked",
  "protected",
  "robust",
  "revert_as",
  "hide_content_reversion",
];

/// Perl `CheckOptions($operation, $allowed, %options)`: one `Error:misdefined` naming every key
/// `allowed` lacks. Returns whether all keys were allowed (`Err` only when the error cap escalates); the definition proceeds either way, as
/// in Perl, with the rejected keys ignored.
pub fn check_options<'a>(
  operation: &str,
  allowed: &[&str],
  keys: impl IntoIterator<Item = &'a str>,
) -> crate::common::error::Result<bool> {
  let bad: Vec<&str> = keys.into_iter().filter(|k| !allowed.contains(k)).collect();
  if bad.is_empty() {
    return Ok(true);
  }
  crate::Error!(
    "misdefined",
    operation,
    format!("{operation} does not accept options:{}", bad.join(", "))
  );
  Ok(false)
}

/// A key Perl accepts whose effect the core cannot carry yet: said, never silently dropped.
pub fn unsupported_option(operation: &str, key: &str) {
  crate::Warn!(
    "unsupported",
    operation,
    format!("{operation}: option {key} is not supported yet; ignored")
  );
}
