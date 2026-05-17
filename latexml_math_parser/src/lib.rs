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
pub use parser::MathParser;
pub use parser::text_form;
pub use util::node_to_grammar_lexemes;
// pub fn parse_math(lexematized: Vec<String>, nodes: Vec<Node>) -> Option<XM> { None }
