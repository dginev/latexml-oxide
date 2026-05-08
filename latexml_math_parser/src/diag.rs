//! Diagnostic emission for `latexml_math_parser`.
//!
//! Mirrors the contract of `latexml_post::diag` and
//! `latexml_core::common::error::Error!` so the converter's log harness
//! aggregates math-parser emissions identically to engine + post stages:
//!
//!   `target = "<class>:<object>"`  →  `Error:<class>:<object> <message>`
//!
//! The math parser uses the same Perl-derived class/object conventions:
//!   * `Error('expected', 'id', …)`         — MathParser.pm:151 (xml:id miss)
//!   * `Error('expected', 'arguments', …)`  — MathParser.pm:1394 (XMApp empty)
//!   * `Error('unexpected', 'nodes', …)`    — MathParser.pm:1580 (structure)
//!   * `Fatal('expected', 'MathGrammar', …)` — MathParser.pm:56 (grammar load)
//!   * `Fatal('malformed', '<XMath>', …)`   — MathParser.pm:280 (bad parent)
//!
//! Like `latexml_post::diag`, we don't reuse `latexml_core::common::error::Error!`
//! because that macro early-returns `Err(LatexmlError)` on max-errors /
//! runaway-loop, and math-parser functions return diverse types
//! (`String`, `Option<…>`, etc.), not `Result<_, LatexmlError>`.

#[macro_export]
macro_rules! log_math_error {
  ($category:expr, $object:expr, $msg:expr) => {
    log::error!(target: &format!("{}:{}", $category, $object), "{}", $msg)
  };
  ($category:expr, $object:expr, $fmt:expr, $($arg:tt)+) => {
    log::error!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  };
}

#[macro_export]
macro_rules! log_math_warn {
  ($category:expr, $object:expr, $msg:expr) => {
    log::warn!(target: &format!("{}:{}", $category, $object), "{}", $msg)
  };
  ($category:expr, $object:expr, $fmt:expr, $($arg:tt)+) => {
    log::warn!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  };
}

#[macro_export]
macro_rules! log_math_info {
  ($category:expr, $object:expr, $msg:expr) => {
    log::info!(target: &format!("{}:{}", $category, $object), "{}", $msg)
  };
  ($category:expr, $object:expr, $fmt:expr, $($arg:tt)+) => {
    log::info!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  };
}
