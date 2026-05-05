use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;

use crate::common::error::Result;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::definition::register::RegisterType;
use crate::mouth;
use crate::tokens::Tokens;

static TRAILING_ZEROS: Lazy<Regex> = Lazy::new(|| Regex::new(r"0+$").unwrap());

//======================================================================
// Strictly speaking, Float isn't part of TeX, but it's handy.

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Float(pub f64);

impl Default for Float {
  fn default() -> Self { Float(0.0) }
}

impl Object for Float {
  fn revert(&self) -> Result<Tokens> { Ok(Tokens::new(ExplodeText!(&self.to_string()))) }
  fn stringify(&self) -> String { s!("Float[{}]", self.0) }
  fn be_digested(self) -> Result<crate::Digested> {
    // Float can be digested as a text box containing its string representation
    let s = self.to_string();
    Ok(
      crate::Tbox::new(
        // `pin` takes `AsRef<str>` — `&String` borrows cleanly without
        // an extra `.to_string()` clone that `into_pin` would force.
        crate::common::arena::pin(&s),
        None,
        None,
        Tokens::new(ExplodeText!(&s)),
        crate::common::arena::SymHashMap::default(),
      )
      .into(),
    )
  }
}

impl NumericOps for Float {
  fn new(number: i64) -> Self { Float(number as f64) }
  fn new_f64(number: f64) -> Self { Float(number) }
  fn value_of(self) -> i64 { self.0 as i64 }
  fn value_f64(self) -> f64 { self.0 }
  fn negate(self) -> Self { Float(-self.0) }
  fn register_type(&self) -> RegisterType { RegisterType::Number }
  fn add<T: NumericOps>(self, other: T) -> Self { Float::new_f64(self.0 + other.value_f64()) }
  fn subtract<T: NumericOps>(self, other: T) -> Self { Float::new_f64(self.0 - other.value_f64()) }
  fn multiply<T: NumericOps>(self, other: T) -> Self { Float::new_f64(self.0 * other.value_f64()) }
  fn divide<T: NumericOps>(self, other: T) -> Self { Float::new_f64(self.0 / other.value_f64()) }
}

impl From<Float> for Tokens {
  fn from(v: Float) -> Tokens { mouth::tokenize_internal(&v.to_string()) }
}

impl From<Float> for Option<Tokens> {
  fn from(v: Float) -> Option<Tokens> { Some(v.into()) }
}

impl fmt::Display for Float {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", floatformat(self.0)) }
}

impl Float {
  /// Tight formatting of floats, where we emit them as integers when they do not have a decimal
  /// part used in e.g. the multido.sty binding and test
  pub fn to_tight_string(&self) -> String { custom_float_format(self.0, true) }
}

/// Utility for formatting sane numbers.
pub fn floatformat(n: f64) -> String { custom_float_format(n, false) }
pub fn custom_float_format(n: f64, tight: bool) -> String {
  let mut s = format!("{:.5}", n);
  if s.contains('.') {
    s = TRAILING_ZEROS.replace(&s, "").to_string();
  }
  if s.ends_with('.') {
    if tight {
      // tight format does not need the trailing dot
      s.pop();
    } else {
      s.push('0'); //  Seems TeX prints .0 which in odd corner cases, people use?
    }
  }
  s
}

impl From<&str> for Float {
  /// Non-numeric input silently becomes 0.0 (Perl parity — see From<String> impl).
  fn from(spec: &str) -> Self { Float(spec.trim().parse::<f64>().unwrap_or(0.0)) }
}
impl From<String> for Float {
  /// Parse a string into a Float. Non-numeric input silently becomes 0.0
  /// to match Perl's implicit numeric coercion — `Float("abc")` + x in
  /// Perl yields x, not a panic.
  fn from(spec: String) -> Self { Float(spec.trim().parse::<f64>().unwrap_or(0.0)) }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn floatformat_integer_gets_dot_zero() {
    assert_eq!(floatformat(1.0), "1.0");
    assert_eq!(floatformat(0.0), "0.0");
    assert_eq!(floatformat(-5.0), "-5.0");
  }

  #[test]
  fn floatformat_trims_trailing_zeros() {
    assert_eq!(floatformat(1.5), "1.5");
    assert_eq!(floatformat(1.25), "1.25");
    assert_eq!(floatformat(0.10000), "0.1");
  }

  #[test]
  fn tight_format_drops_dot_for_integers() {
    assert_eq!(Float(1.0).to_tight_string(), "1");
    assert_eq!(Float(0.0).to_tight_string(), "0");
    assert_eq!(Float(1.5).to_tight_string(), "1.5");
  }

  #[test]
  fn custom_float_format_precision() {
    let out = custom_float_format(0.123456789, false);
    assert!(
      out.starts_with("0.12346") || out.starts_with("0.12345"),
      "got {out:?}"
    );
  }

  #[test]
  fn from_str_nonnumeric_is_zero() {
    assert_eq!(Float::from("abc").0, 0.0);
    assert_eq!(Float::from("  xyz  ").0, 0.0);
    assert_eq!(Float::from("").0, 0.0);
  }

  #[test]
  fn from_str_numeric_parses() {
    assert_eq!(Float::from("1.5").0, 1.5);
    assert_eq!(Float::from("  -3.125  ").0, -3.125);
    assert_eq!(Float::from("42").0, 42.0);
  }

  #[test]
  fn from_string_matches_from_str() {
    for s in &["1", "1.5", "", "abc", "-0.0001"] {
      let a = Float::from(*s).0;
      let b = Float::from(s.to_string()).0;
      assert_eq!(a, b, "divergence on {s:?}: {a} vs {b}");
    }
  }

  #[test]
  fn float_arithmetic_roundtrip() {
    let a = Float::new_f64(1.5);
    let b = Float::new_f64(2.5);
    assert_eq!(a.add(b).value_f64(), 4.0);
    assert_eq!(b.subtract(a).value_f64(), 1.0);
    assert_eq!(a.multiply(b).value_f64(), 3.75);
    assert_eq!(b.divide(a).value_f64(), 2.5 / 1.5);
  }

  #[test]
  fn float_negate() {
    assert_eq!(Float::new_f64(1.5).negate().value_f64(), -1.5);
    assert_eq!(Float::new_f64(0.0).negate().value_f64(), 0.0);
  }
}
