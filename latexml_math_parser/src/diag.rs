//! Diagnostic emission for `latexml_math_parser`.
//!
//! Thin forwarding wrappers over the SINGLE diagnostic vehicle
//! (`latexml_core::common::error::emit_*`), so math-parser emissions count in
//! the conversion tally and aggregate in cortex identically to engine + post
//! stages:
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
//! We don't reuse `latexml_core::common::error::Error!` because that macro
//! early-returns `Err(LatexmlError)` on max-errors / runaway-loop, and
//! math-parser functions return diverse types (`String`, `Option<…>`, etc.),
//! not `Result<_, LatexmlError>` — exactly the gap `emit_error` exists for.
//! These used to call raw `log::warn!`/`log::error!`, which printed
//! `Warning:`/`Error:` lines that no counter ever saw: the 131 MB witness
//! logged 12,103 math warnings and reported "2 warnings".

#[macro_export]
macro_rules! log_math_error {
  ($category:expr_2021, $object:expr_2021, $msg:expr_2021) => {
    latexml_core::common::error::emit_error(&format!("{}", $category), &format!("{}", $object), &format!("{}", $msg))
  };
  ($category:expr_2021, $object:expr_2021, $fmt:expr_2021, $($arg:tt)+) => {
    latexml_core::common::error::emit_error(&format!("{}", $category), &format!("{}", $object), &format!($fmt, $($arg)+))
  };
}

#[macro_export]
macro_rules! log_math_warn {
  ($category:expr_2021, $object:expr_2021, $msg:expr_2021) => {
    latexml_core::common::error::emit_warn(&format!("{}", $category), &format!("{}", $object), &format!("{}", $msg))
  };
  ($category:expr_2021, $object:expr_2021, $fmt:expr_2021, $($arg:tt)+) => {
    latexml_core::common::error::emit_warn(&format!("{}", $category), &format!("{}", $object), &format!($fmt, $($arg)+))
  };
}

#[macro_export]
macro_rules! log_math_info {
  ($category:expr_2021, $object:expr_2021, $msg:expr_2021) => {
    latexml_core::common::error::emit_info(&format!("{}", $category), &format!("{}", $object), &format!("{}", $msg))
  };
  ($category:expr_2021, $object:expr_2021, $fmt:expr_2021, $($arg:tt)+) => {
    latexml_core::common::error::emit_info(&format!("{}", $category), &format!("{}", $object), &format!($fmt, $($arg)+))
  };
}
