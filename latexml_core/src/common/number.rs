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
  fn be_digested(self) -> Result<Digested> { Ok(Digested::from(RegisterValue::Number(self))) }
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn number_new_and_value_of() {
    assert_eq!(Number::new(42).value_of(), 42);
    assert_eq!(Number::new(-1).value_of(), -1);
    assert_eq!(Number::new(0).value_of(), 0);
  }

  #[test]
  fn number_new_f64_truncates() {
    // new_f64 truncates (not rounds).
    assert_eq!(Number::new_f64(3.7).value_of(), 3);
    assert_eq!(Number::new_f64(-3.7).value_of(), -3);
    assert_eq!(Number::new_f64(0.0).value_of(), 0);
  }

  #[test]
  fn number_from_string_numeric() {
    assert_eq!(Number::from(String::from("42")).value_of(), 42);
    assert_eq!(Number::from(String::from("-7")).value_of(), -7);
    assert_eq!(Number::from(String::from("  100  ")).value_of(), 100);
  }

  #[test]
  fn number_from_string_nonnumeric_is_zero() {
    // Perl-parity: non-numeric → 0 silently.
    assert_eq!(Number::from(String::from("abc")).value_of(), 0);
    assert_eq!(Number::from(String::from("")).value_of(), 0);
    assert_eq!(
      Number::from(String::from("3.14")).value_of(),
      0,
      "float string doesn't parse as integer — coerces to 0"
    );
  }

  #[test]
  fn number_display() {
    assert_eq!(format!("{}", Number::new(42)), "42");
    assert_eq!(format!("{}", Number::new(-7)), "-7");
    assert_eq!(format!("{}", Number::new(0)), "0");
  }

  #[test]
  fn number_to_attribute() {
    assert_eq!(Number::new(42).to_attribute(), "42");
    assert_eq!(Number::new(-7).to_attribute(), "-7");
  }

  #[test]
  fn number_default_is_zero() {
    assert_eq!(Number::default().value_of(), 0);
  }

  #[test]
  fn number_equality() {
    assert_eq!(Number::new(42), Number::new(42));
    assert_ne!(Number::new(42), Number::new(43));
  }

  #[test]
  fn number_from_catcode_matches_u8() {
    // From<Catcode> coerces to the catcode's u8 value.
    assert_eq!(
      Number::from(Catcode::ESCAPE).value_of(),
      u8::from(Catcode::ESCAPE) as i64
    );
    assert_eq!(
      Number::from(Catcode::LETTER).value_of(),
      u8::from(Catcode::LETTER) as i64
    );
  }

  // Note: the Number!() macro references `::latexml_core::...` and
  // so can only be tested from external integration tests. Not
  // covered here.
}
