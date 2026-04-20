use crate::Object;
use crate::common::error::*;
use crate::common::numeric_ops::NumericOps;
use crate::definition::register::RegisterType;
use crate::definition::register::RegisterValue;
use crate::digested::Digested;
use crate::mouth;
use crate::token::Catcode;
use crate::tokens::Tokens;
use std::fmt;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Number(pub i64);
impl Object for Number {
  fn revert(&self) -> Result<Tokens> { Ok(Tokens::new(ExplodeText!(&self.0.to_string()))) }
  // Perl: beDigested returns $self (duck typing). In Rust, wrap as RegisterValue.
  fn be_digested(self) -> Result<Digested> {
    Ok(Digested::from(RegisterValue::Number(self)))
  }
}
impl NumericOps for Number {
  fn new(number: i64) -> Self { Number(number) }
  fn new_f64(number: f64) -> Self { Number(number.trunc() as i64) }
  fn value_of(self) -> i64 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::Number }
}

impl Number {
  pub fn to_attribute(&self) -> String { self.0.to_string() }
}

impl From<Number> for Tokens {
  fn from(v: Number) -> Tokens { mouth::tokenize_internal(&v.0.to_string()) }
}

impl From<Number> for Option<Tokens> {
  fn from(v: Number) -> Option<Tokens> { Some(v.into()) }
}

#[macro_export]
macro_rules! Number {
  ($number:expr) => {{ ::latexml_core::common::number::Number::new($number as i64) }};
}

impl From<String> for Number {
  /// Parse a string into a Number. Non-numeric input silently becomes 0
  /// to match Perl's `Number(ToString($x))` coercion — Perl's implicit
  /// numeric context treats "abc" / undef as 0 without panicking.
  fn from(s: String) -> Number { Number(s.trim().parse::<i64>().unwrap_or(0)) }
}

impl fmt::Display for Number {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

impl From<Catcode> for Number {
  fn from(c: Catcode) -> Number { Number::new(u8::from(c) as i64) }
}
