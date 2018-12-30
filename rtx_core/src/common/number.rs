use crate::token::{Catcode, Token};
use std::borrow::Cow;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Number {
  number: f32,
}

impl Default for Number {
  fn default() -> Self { Number::new(0.0) }
}

impl Number {
  pub fn new<T: Into<f32>>(number: T) -> Self { Number { number: number.into() } }
  pub fn value_of(self) -> f32 { self.number }
  pub fn add(self, other: Number) -> Self { Number::new(self.value_of() + other.value_of()) }
  pub fn negate(self) -> Number {
    if self.number > 0.0 {
      Number::new(-self.number)
    } else {
      self
    }
  }
  pub fn to_token(self) -> Token { T_OTHER!(self.number.to_string()) }
}

#[macro_export]
macro_rules! Number {
  ($number:expr) => {
    ::rtx_core::common::number::Number::new($number as f32)
  };
}
