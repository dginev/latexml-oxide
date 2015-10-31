use core::token::Token;

/// Box is a Rust keyword, so we use "TBox" instead, as in "TeX Box"
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