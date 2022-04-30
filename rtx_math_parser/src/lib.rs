#[macro_use]
mod grammar;
mod data;
mod parser;
mod pragmatics;
mod semantics;
mod syntax;
mod util;

pub use parser::MathParser;
pub use util::node_to_grammar_lexemes;
pub use parser::text_form;
// pub fn parse_math(lexematized: Vec<String>, nodes: Vec<Node>) -> Option<Tree> { None }
