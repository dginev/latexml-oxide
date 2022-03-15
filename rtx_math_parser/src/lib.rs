#[macro_use]
mod grammar;
mod syntax;
mod semantics;
mod pragmatics;
mod util;

use semantics::{Tree};

pub fn tokenize_tex(tex:&str) -> Vec<String> {
  Vec::new()
}

pub fn parse_math(tokens: Vec<String>) -> Option<Tree> {
  None
}
