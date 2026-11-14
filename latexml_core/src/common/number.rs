use std::fmt;

use crate::{
  Object,
  common::{error::*, numeric_ops::NumericOps},
  definition::register::{RegisterType, RegisterValue},
  digested::Digested,
  mouth,
  token::Catcode,
  tokens::{TeXString, Tokens},
};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Number(pub i64);
impl Object for Number {
  fn revert(&self) -> Result<Tokens> { Ok(Tokens::new(ExplodeText!(&self.0.to_string()))) }
  // Perl: beDigested returns $self (duck typing). In Rust, wrap as RegisterValue.
  fn be_digested(self) -> Result<Digested> { Ok(Digested::from(RegisterValue::Number(self))) }
}
impl NumericOps for Number {
  fn arith_ceiling() -> i64 { crate::common::numeric_ops::MAX_INTEGER }
  fn new(number: i64) -> Self { Number(number) }
  fn new_f64(number: f64) -> Self { Number(number.trunc() as i64) }
  fn value_of(self) -> i64 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::Number }
}

impl Number {
  pub fn to_attribute(&self) -> String { self.0.to_string() }
}

impl From<Number> for Tokens {
  fn from(v: Number) -> Tokens { mouth::tokenize_internal(TeXString::assembled(v.0.to_string())) }
}

impl From<Number> for Option<Tokens> {
  fn from(v: Number) -> Option<Tokens> { Some(v.into()) }
}

#[macro_export]
macro_rules! Number {
  ($number:expr_2021) => {{ ::latexml_core::common::number::Number::new($number as i64) }};
}

impl From<String> for Number {
  fn from(s: String) -> Number { Number::from_str_value(&s) }
}

impl Number {
  /// Perl Number.pm:31-34 `int($number)`: Perl's numeric coercion, truncated toward zero (`"1e3"` is 1000, `"1.5"` 1);
  /// a string that is not wholly a number Warns as Perl's `isn't numeric` and reads its numeric prefix (`"12abc"` 12).
  pub fn from_str_value(spec: &str) -> Number {
    let trimmed = spec.trim();
    let prefix = numeric_prefix(trimmed);
    if prefix.is_empty() || prefix.len() != trimmed.len() {
      Warn!(
        "perl",
        "warn",
        s!("Argument \"{spec}\" isn't numeric in int")
      );
    }
    Number(prefix.parse::<f64>().map_or(0, |f| f.trunc() as i64))
  }
}

/// The longest prefix of `s` Perl's string-to-number conversion reads: `[+-]?\d*(\.\d*)?([eE][+-]?\d+)?`.
fn numeric_prefix(s: &str) -> &str {
  let b = s.as_bytes();
  let digits = |mut i: usize| {
    while i < b.len() && b[i].is_ascii_digit() {
      i += 1;
    }
    i
  };
  let mut i = usize::from(matches!(b.first(), Some(b'+' | b'-')));
  let start = i;
  i = digits(i);
  let mut mantissa = i > start;
  if b.get(i) == Some(&b'.') {
    let j = digits(i + 1);
    mantissa |= j > i + 1;
    i = j;
  }
  if !mantissa {
    return "";
  }
  if matches!(b.get(i), Some(b'e' | b'E')) {
    let k = i + 1 + usize::from(matches!(b.get(i + 1), Some(b'+' | b'-')));
    let e = digits(k);
    if e > k {
      i = e;
    }
  }
  &s[..i]
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
  fn number_from_string_is_perls_int() {
    // Perl `int($s)` (Number.pm:31-34): numeric coercion truncated; a non-numeric remainder reads the prefix.
    for (input, n) in [
      ("abc", 0),
      ("", 0),
      (".5", 0),
      ("3.14", 3),
      ("1e3", 1000),
      ("-7", -7),
      (" 42 ", 42),
      ("12abc", 12),
    ] {
      assert_eq!(Number::from(String::from(input)).value_of(), n, "{input}");
    }
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
