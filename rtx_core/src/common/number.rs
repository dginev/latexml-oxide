use crate::definition::register::NumericOps;
use crate::token::{Catcode, Token};
use std::borrow::Cow;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Number {
  number: f32,
}

impl Default for Number {
  fn default() -> Self { Number::new(0.0) }
}

impl NumericOps for Number {
  fn new<T: Into<f32>>(number: T) -> Self { Number { number: number.into() } }
  fn value_of(self) -> f32 { self.number }
}
impl Number {
  pub fn to_token(self) -> Token { T_OTHER!(self.number.to_string()) }
}

#[macro_export]
macro_rules! Number {
  ($number:expr) => {{
    use ::rtx_core::definition::register::NumericOps;
    ::rtx_core::common::number::Number::new($number as f32)
  }};
}
