use crate::common::glue::Glue;
use crate::definition::register::RegisterType;
use crate::token::{Catcode, Token};
use std::borrow::Cow;

pub const UNITY: i32 = 65536;
pub const EPSILON: f32 = 0.000_000_119_209_29;
pub const ROUNDING_HALF: f32 = 0.49999994;
pub const SCALES: &[i32] = &[1, 10, 100, 1000, 10000, 100_000];

/// Round $number to $prec decimals (0...6) attempting to do so portably.
pub fn round_to(number: f32, prec_opt: Option<u8>) -> f32 {
  let mut prec = prec_opt.unwrap_or(2);
  if prec > 5 {
    prec = 5;
  }
  let scale = SCALES[prec as usize];
  // scale to integer, w/some slop in case arbitrarily close to an integer...
  let n = number * scale as f32 * (1.0 + 100.0 * EPSILON);
  let adjusted: f32 = if n < -EPSILON {
    n - 0.5
  } else if n > EPSILON {
    n + 0.5
  } else {
    0.0
  };
  adjusted.trunc() / scale as f32
}

/// An attempt at rounding floats to integers (like scaled points),
/// in a (hopefully) Knuthian manner (like round_decimals \S102 in Tex The Program)
pub fn kround(number: f32) -> i32 {
  let rounded = if number < 0.0 { number - ROUNDING_HALF } else { number + ROUNDING_HALF };
  rounded.trunc() as i32
}

/// Convert `float` to a fixed-point number
/// If `unit` is given, it is number of units PER SCALED-POINT! (hence, extra division)
/// AND, note that the float is rounded and THEN truncated after multiplying by units!
/// to mimic TeX's behavior.
pub fn fixpoint(float: f32, unit_opt: Option<f32>) -> i32 {
  let fix = kround(float * UNITY as f32);
  if let Some(unit) = unit_opt {
    (fix as f32 * unit / UNITY as f32).trunc() as i32
  } else {
    fix
  }
}

pub trait NumericOps {
  fn new(num: i32) -> Self
  where Self: Sized;
  fn new_f32(num: f32) -> Self
  where Self: Sized;
  fn unit(&self) -> Option<&'static str> { None }
  fn value_of(self) -> i32;
  fn value_f32(self) -> f32
  where Self: Sized {
    self.value_of() as f32
  }
  fn pt_value(self, prec: Option<u8>) -> f32
  where Self: Sized {
    round_to(self.value_of() as f32 / UNITY as f32, prec)
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
  fn divide<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    let mut other_value: f32 = other.value_of() as f32;
    if other_value == 0.0 {
      other_value = EPSILON; // avoid dividing by zero
    }
    Self::new((self.value_of() as f32 / other_value).trunc() as i32)
  }

  fn to_token(self) -> Token
  where Self: Sized {
    T_OTHER!(self.value_of().to_string())
  }
  // dancing around meta-programming in the Glue case... is there a better way?
  fn into_glue_type(self) -> Glue
  where Self: Sized {
    unimplemented!()
  }
  fn register_type(&self) -> RegisterType;
}
