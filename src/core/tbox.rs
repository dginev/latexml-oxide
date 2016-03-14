use core::token::Token;
use core::Digested;

/// Box is a Rust keyword, so we use "TBox" instead, as in "TeX Box"
#[derive(Debug)]
pub struct TBox {//TODO
   pub text : String,
   pub font : String,
   pub locator : String,
   pub tokens : Vec<Token>
}

pub fn TBox() -> TBox {
  TBox {
    text: String::new(),
    font: String::new(),
    locator: String::new(),
    tokens : Vec::new()
  }
}

impl Digested for TBox {
  fn to_string(&self) -> String {
    self.text.clone()
  }
  fn unlist(&self) -> Vec<&TBox> {
    vec![self]
  }
}