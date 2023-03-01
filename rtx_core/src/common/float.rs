use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::fmt;

use crate::common::locator::Locator;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::common::error::Result;
use crate::definition::register::RegisterType;
use crate::mouth;
use crate::state::State;
use crate::tokens::Tokens;

lazy_static! {
  static ref TRAILING_ZEROS: Regex = Regex::new(r"0+$").unwrap();
}

//======================================================================
// Strictly speaking, Float isn't part of TeX, but it's handy.

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Float(pub f32);

impl Default for Float {
  fn default() -> Self { Float(0.0) }
}

impl Object for Float {
  fn get_locator(&self) -> Option<Cow<Locator>> { None }
  fn revert(&self, state: &State) -> Result<Tokens> { Ok(Tokens::new(ExplodeText!(&self.0.to_string()))) }
  fn stringify(&self) -> String { s!("Float[{}]", self.0) }
}
impl NumericOps for Float {
  fn new(number: i64) -> Self { Float(number as f32) }
  fn new_f32(number: f32) -> Self { Float(number) }
  fn value_of(self) -> i64 { self.0 as i64 }
  fn value_f32(self) -> f32 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::Number }
  fn add<T: NumericOps>(self, other: T) -> Self { Float::new_f32(self.0 + other.value_f32()) }
  fn subtract<T: NumericOps>(self, other: T) -> Self { Float::new_f32(self.0 - other.value_f32()) }
  fn multiply<T: NumericOps>(self, other: T) -> Self { Float::new_f32(self.value_f32() * other.value_f32()) }
  fn divide<T: NumericOps>(self, other: T) -> Self { Float::new_f32(self.value_f32() / other.value_f32()) }
}

impl From<Float> for Tokens {
  fn from(v: Float) -> Tokens { mouth::tokenize_internal(&v.to_string()) }
}

impl From<Float> for Option<Tokens> {
  fn from(v: Float) -> Option<Tokens> { Some(v.into()) }
}

impl fmt::Display for Float {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

/// Utility for formatting sane numbers.
pub fn floatformat(n: f32) -> String {
  let mut s = s!("{:.5}", n);
  if s.contains('.') {
    s = TRAILING_ZEROS.replace(&s, "").to_string();
  }
  if s.ends_with('.') {
    s.push('0'); //  Seems TeX prints .0 which in odd corner cases, people use?
  }
  s
}

impl From<&str> for Float {
  fn from(spec:&str) -> Self {
    Float(spec.parse::<f32>().expect("Float::from(&str) does not handle malformed spec strings"))
  }
}
impl From<String> for Float {
  fn from(spec:String) -> Self {
    Float(spec.parse::<f32>().expect("Float::from(String) does not handle malformed spec strings"))
  }
}
