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
        None, None,
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
  fn from(spec: &str) -> Self {
    Float(
      spec
        .parse::<f64>()
        .expect("Float::from(&str) does not handle malformed spec strings"),
    )
  }
}
impl From<String> for Float {
  /// Parse a string into a Float. Non-numeric input silently becomes 0.0
  /// to match Perl's implicit numeric coercion — `Float("abc")` + x in
  /// Perl yields x, not a panic.
  fn from(spec: String) -> Self { Float(spec.trim().parse::<f64>().unwrap_or(0.0)) }
}
