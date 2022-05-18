use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;
use std::borrow::Cow;

use crate::common::numeric_ops::NumericOps;
use crate::common::locator::Locator;
use crate::definition::register::{RegisterType};
use crate::mouth;
use crate::common::object::Object;
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
  fn get_locator(&self) -> Option<Cow<Locator>> {
    None
  }
}
impl NumericOps for Float {
  fn new(number: i32) -> Self { Float(number as f32) }
  fn new_f32(number: f32) -> Self { Float(number) }
  fn value_of(self) -> i32 { self.0 as i32 }
  fn register_type(&self) -> RegisterType { RegisterType::Number }
  fn add<T: NumericOps>(self, other: T) -> Self { Float::new_f32(self.0 + other.value_f32()) }
  fn subtract<T: NumericOps>(self, other: T) -> Self { Float::new_f32(self.0 - other.value_f32()) }
}

impl From<Float> for Tokens {
  fn from(v: Float) -> Tokens { mouth::tokenize_internal(&v.to_string(), None) }
}

impl From<Float> for Option<Tokens> {
  fn from(v: Float) -> Option<Tokens> { Some(v.into()) }
}

impl fmt::Display for Float {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

impl Float {
  fn value_of(self) -> i32 { self.0 as i32 }
  pub fn multiply(&self, other: &Float) -> Self { Float::new(self.value_of() * other.value_of()) }

  pub fn stringify(&self) -> String { s!("Float[{}]", self.0) }
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
