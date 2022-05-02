use std::fmt;
use lazy_static::lazy_static;
use regex::Regex;

use crate::common::number::kround;
use crate::common::dimension::{fixpoint,UNITY};
use crate::definition::register::{NumericOps, RegisterType};
use super::dimension::Dimension;

lazy_static! {
  static ref SPEC_RE : Regex = Regex::new(r"^(-?\d*\.?\d*)mu$").unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MuDimension(pub f32);

impl NumericOps for MuDimension {
  fn value_of(self) -> f32 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::MuDimension }
}

impl Default for MuDimension {
  fn default() -> Self { MuDimension(0.0) }
}

impl fmt::Display for MuDimension {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", Dimension::point_format(self.0)) }
}

impl MuDimension {
  pub fn new<T: Into<f32>>(number: T) -> Self { MuDimension(number.into()) }
  pub fn new_spec(spec: &str) -> Self {
    if let Some(cap) = SPEC_RE.captures(spec) {
      MuDimension(fixpoint(cap.get(1).map_or("", |m| m.as_str()).parse::<f32>().unwrap(), Some(UNITY as f32)))
    } else {
      MuDimension(kround(spec.parse::<f32>().unwrap()))
    }
  }

  pub fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() + other.value_of())
  }
  pub fn negate(self) -> Self
  where Self: Sized {
    let value = self.value_of();
    if value > 0.0 {
      Self::new(-value)
    } else {
      Self::new(value)
    }
  }
  pub fn multiply<T: Into<f32>>(self, other: T) -> Self
  where Self: Sized {
    let other: f32 = other.into();
    Self::new((self.value_of() * other).floor())
  }
  pub fn divide<T: Into<f32>>(self, other: T) -> Self
  where Self: Sized {
    let other: f32 = other.into();
    Self::new((self.value_of() / other).floor())
  }
}