use std::fmt;
use crate::definition::register::{NumericOps, RegisterType};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Number(pub f32);

impl Default for Number {
  fn default() -> Self { Number(0.0) }
}

impl NumericOps for Number {
  fn new<T: Into<f32>>(number: T) -> Self { Number(number.into()) }
  fn value_of(self) -> f32 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::Number }
}

#[macro_export]
macro_rules! Number {
  ($number:expr) => {{
    use ::rtx_core::definition::register::NumericOps;
    ::rtx_core::common::number::Number::new($number as f32)
  }};
}

impl From<String> for Number {
  fn from(s: String) -> Number { Number(s.parse::<f32>().unwrap()) }
}

impl fmt::Display for Number {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}