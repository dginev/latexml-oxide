extern crate rustc_hash;

#[macro_use]
mod grammar;
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
