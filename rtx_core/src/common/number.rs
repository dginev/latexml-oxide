use crate::definition::register::{NumericOps, RegisterType};
use crate::mouth;
use crate::tokens::Tokens;
use crate::{Locator, Object};
use lazy_static::lazy_static;
use std::borrow::Cow;
use std::fmt;

lazy_static! {
  /// smallest number that makes a difference added to 1 in Perl's float format.
  static ref EPSILON : f32 = {
    let mut e = 1.0;
    while 1.0 + e / 2.0 != 1.0 {
      e /= 2.0;
    }
    e
  };

  static ref ROUNDING_HALF : f32 = 0.5 * (1.0 - *EPSILON);
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Number(pub i32);
impl Object for Number {
  fn get_locator(&self) -> Option<Cow<Locator>> { None }
}
impl NumericOps for Number {
  fn value_of(self) -> f32 { self.0 as f32 }
  fn register_type(&self) -> RegisterType { RegisterType::Number }
  fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_i32() + other.value_of() as i32)
  }
  fn subtract<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_i32() - other.value_of() as i32)
  }
}

impl Number {
  pub fn new(number: i32) -> Self { Number(number) }
  pub fn new_f32(number: f32) -> Self { Number(number.trunc() as i32) }
  pub fn value_i32(self) -> i32 { self.0 }
  pub fn to_attribute(&self) -> String { self.0.to_string() }
  pub fn negate(self) -> Self
  where Self: Sized {
    let value = self.value_i32();
    if value > 0 {
      Self::new(-value)
    } else {
      Self::new(value)
    }
  }

  pub fn multiply<T: Into<f32>>(self, other: T) -> Self
  where Self: Sized {
    let other: f32 = other.into();
    Self::new((self.value_of() * other).trunc() as i32)
  }
  pub fn divide<T: Into<f32>>(self, other: T) -> Self
  where Self: Sized {
    let mut other: f32 = other.into();
    if other == 0.0 {
      other = *EPSILON; // avoid dividing by zero
    }
    Self::new((self.value_of() / other).trunc() as i32)
  }
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

/// An attempt at rounding floats to integers (like scaled points),
/// in a (hopefully) Knuthian manner (like round_decimals \S102 in Tex The Program)
pub fn kround(number: f32) -> i32 {
  let rounded = if number < 0.0 {
    number - *ROUNDING_HALF
  } else {
    number + *ROUNDING_HALF
  };
  rounded.trunc() as i32
}
