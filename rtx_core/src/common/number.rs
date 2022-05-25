use crate::common::numeric_ops::NumericOps;
use crate::definition::register::{RegisterType};
use crate::mouth;
use crate::tokens::Tokens;
use crate::{Locator, Object};
use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Number(pub i32);
impl Object for Number {
  fn get_locator(&self) -> Option<Cow<Locator>> { None }
}
impl NumericOps for Number {
  fn new(number: i32) -> Self { Number(number) }
  fn new_f32(number: f32) -> Self { Number(number.trunc() as i32) }
  fn value_of(self) -> i32 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::Number }
}

impl Number {
  pub fn to_attribute(&self) -> String { self.0.to_string() }
}

impl From<Number> for Tokens {
  fn from(v: Number) -> Tokens { mouth::tokenize_internal(&v.to_string(), None) }
}

impl From<Number> for Option<Tokens> {
  fn from(v: Number) -> Option<Tokens> { Some(v.into()) }
}

#[macro_export]
macro_rules! Number {
  ($number:expr) => {{
    ::rtx_core::common::number::Number::new($number as i32)
  }};
}

impl From<String> for Number {
  fn from(s: String) -> Number { Number(s.parse::<i32>().unwrap()) }
}

impl fmt::Display for Number {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}
