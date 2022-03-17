#[macro_use]
mod grammar;
mod syntax;
mod semantics;
mod pragmatics;
mod util;
mod data;
mod parser;

pub use parser::MathParser;
pub use util::node_to_grammar_lexemes;
// pub fn parse_math(lexematized: Vec<String>, nodes: Vec<Node>) -> Option<Tree> { None }
