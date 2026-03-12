use crate::common::glue::Glue;
use crate::definition::register::RegisterType;
use crate::token::{Catcode, Token};
use std::fmt::Display;

pub const UNITY: i64 = 65536;
pub const UNITY_F64: f64 = 65536.0;
pub const EPSILON: f64 = 0.000_000_119_209_29;
pub const ROUNDING_HALF: f64 = 0.49999994;
pub const SCALES: &[i32] = &[1, 10, 100, 1000, 10000, 100_000];

/// Round $number to $prec decimals (0...6) attempting to do so portably.
pub fn round_to(number: f64, prec_opt: Option<u8>) -> f64 {
  let mut prec = prec_opt.unwrap_or(2);
  if prec > 5 {
    prec = 5;
  }
  let scale = SCALES[prec as usize];
  // scale to integer, w/some slop in case arbitrarily close to an integer...
  let n = number * scale as f64 * (1.0 + 100.0 * EPSILON);
  let adjusted: f64 = if n < -EPSILON {
    n - 0.5
  } else if n > EPSILON {
    n + 0.5
  } else {
    0.0
  };
  adjusted.trunc() / scale as f64
}

/// An attempt at rounding floats to integers (like scaled points),
/// in a (hopefully) Knuthian manner (like round_decimals \S102 in Tex The Program)
// DG: Note that we have to go to the largest `i64` type to contain the truncation
// of large SP values multiplied up by UNITY
pub fn kround(number: f64) -> i64 {
  let rounded = if number < 0.0 {
    number - ROUNDING_HALF
  } else {
    number + ROUNDING_HALF
  };
  rounded.trunc() as i64
}

/// Convert `float` to a fixed-point number
///
/// If `unit` is given, it is number of units PER SCALED-POINT! (hence, extra division)
/// AND, note that the float is rounded and THEN truncated after multiplying by units!
/// to mimic TeX's behavior.
pub fn fixpoint(float: f64, unit_opt: Option<f64>) -> i64 {
  let fix = kround(float * UNITY_F64);
  if let Some(unit) = unit_opt {
    (fix as f64 * unit / UNITY_F64).trunc() as i64
  } else {
    fix
  }
}

pub trait NumericOps {
  fn new(num: i64) -> Self
  where Self: Sized;
  fn new_f64(num: f64) -> Self
  where Self: Sized;
  fn unit(&self) -> Option<&'static str> { None }
  fn value_of(self) -> i64;
  fn value_f64(self) -> f64
  where Self: Sized {
    self.value_of() as f64
  }
  fn pt_value(self, prec: Option<u8>) -> f64
  where Self: Sized {
    round_to(self.value_of() as f64 / UNITY_F64, prec)
  }
  fn px_value(self, prec: Option<u8>) -> f64
  where Self: Sized {
    let dpi = crate::state::lookup_int("DPI");
    let dpi = if dpi > 0 { dpi as f64 } else { 100.0 };
    round_to((self.value_f64() / UNITY_F64) * (dpi / 72.27), prec)
  }

  fn absolute(self) -> Self
  where Self: Sized {
    Self::new(self.value_of().abs())
  }

  fn sign(self) -> i8
  where Self: Sized {
    use std::cmp::Ordering::*;
    match self.value_of().cmp(&0) {
      Less => -1,
      Equal => 0,
      Greater => 1,
    }
  }

  fn negate(self) -> Self
  where Self: Sized {
    Self::new(-self.value_of())
  }
  fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() + other.value_of())
  }
  fn subtract<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() - other.value_of())
  }
  fn multiply<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() * other.value_of())
  }
  /// Truncating division
  fn divide<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    let mut other_value: f64 = other.value_of() as f64;
    if other_value == 0.0 {
      other_value = EPSILON; // avoid dividing by zero
    }
    Self::new((self.value_of() as f64 / other_value).trunc() as i64)
  }

  /// Rounding division
  fn divideround<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    let mut other_value: f64 = other.value_of() as f64;
    if other_value == 0.0 {
      other_value = EPSILON; // avoid dividing by zero
    }
    Self::new((0.5 + self.value_of() as f64 / other_value).trunc() as i64)
  }

  fn smaller<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of().min(other.value_of()))
  }

  fn larger<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of().max(other.value_of()))
  }

  fn to_token(self) -> Token
  where Self: Sized {
    T_OTHER!(self.value_of().to_string())
  }
  // dancing around meta-programming in the Glue case... is there a better way?
  fn into_glue_type(self) -> Glue
  where Self: Sized {
    todo!()
  }
  fn register_type(&self) -> RegisterType;
  fn to_attribute(&self) -> String
  where Self: Display {
    self.to_string()
  }
}
