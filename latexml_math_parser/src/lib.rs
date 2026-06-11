// semantics/tree.rs is under active development and carries intentional
// `todo!()` stubs for not-yet-handled XM node kinds. The workspace lint policy
// warns on `todo!()` (denied in CI via -D warnings); scope an allow to this
// in-progress crate so the math parser is not gated on its own WIP. Remove once
// the XMRef/XMArg semantics land.
#![allow(clippy::todo)]

extern crate rustc_hash;

// Crate-wide diagnostic emission macros (`log_math_error!` /
// `log_math_warn!` / `log_math_info!`). Loaded first via #[macro_use]
// so every math_parser module can use them without explicit imports.
// Mirrors latexml_post's diag.rs — emits structured
// `Error:<class>:<object>` lines for harness aggregation.
#[macro_use]
mod diag;

#[macro_use]
mod grammar;
mod asf_traverser;
mod data;
mod parser;
mod pragmatics;
mod semantics;
mod util;

pub use data::get_grammatical_role;
pub use parser::{MathParser, text_form};
pub use util::node_to_grammar_lexemes;

/// Print and reset the thread-local Marpa ASF instrumentation
/// counters (codex's `MARPA_ASF_STATS=1` plan from
/// `marpa/docs/ASF_PERFORMANCE_FINDINGS.md`). No-op when the env
/// var is unset.
///
/// Intended to be called once per document conversion from the
/// `latexml_oxide` converter so each `.tex → .html` run emits at
/// most one stats line; aggregation across a corpus is done
/// offline by piping stderr through `grep MARPA_ASF_STATS`.
pub fn report_and_reset_asf_stats() {
  if marpa::asf::asf_stats_enabled()
    && let Some(snapshot) = marpa::asf::snapshot()
  {
    eprintln!("{}", snapshot.as_log_line());
    marpa::asf::reset();
  }
}
