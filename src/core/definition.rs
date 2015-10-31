use core::gullet::Gullet;
use core::token::Token;

#[derive(Clone)]
pub struct Definition {
    pub is_expandable : bool,
    pub is_protected : bool,

}

impl Definition {
  pub fn invoke(&mut self, gullet : &mut Gullet) -> Vec<Token> {
    Vec::new()
  }
}