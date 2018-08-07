use token::Token;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Number {
  number: i32,
}

impl Number {
  pub fn new(number: i32) -> Self { Number { number: number } }
  pub fn value_of(&self) -> i32 { self.number }
  pub fn add(&self, other: Number) -> Self { Number::new(self.value_of() + other.value_of()) }
  pub fn negate(&self) -> Number {
    if self.number > 0 {
      Number::new(-self.number)
    } else {
      self.clone()
    }
  }
  pub fn to_token(&self) -> Token { T_OTHER!(self.number.to_string()) }
}

#[macro_export]
macro_rules! Number {
  ($number:expr) => {
    ::rtx_core::common::number::Number::new($number)
  };
}
