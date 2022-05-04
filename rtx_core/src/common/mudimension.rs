use std::fmt;
use std::borrow::Cow;
use lazy_static::lazy_static;
use regex::Regex;

use crate::common::number::kround;
use crate::common::dimension::{fixpoint,UNITY};
use crate::definition::register::{NumericOps, RegisterType};
use crate::{Locator,Object};
use super::dimension::Dimension;

lazy_static! {
  static ref SPEC_RE : Regex = Regex::new(r"^(-?\d*\.?\d*)mu$").unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MuDimension(pub f32);

impl NumericOps for MuDimension {
  fn value_of(self) -> f32 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::MuDimension }
  fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() + other.value_of())
  }
  fn subtract<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() - other.value_of())
  }
}

impl Default for MuDimension {
  fn default() -> Self { MuDimension(0.0) }
}

impl fmt::Display for MuDimension {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", Dimension::point_format(self.0)) }
}
impl Object for MuDimension {
  fn get_locator(&self) -> Option<Cow<Locator>> { None }
}

impl MuDimension {
  pub fn new(number: f32) -> Self { MuDimension(kround(number) as f32) }
  pub fn new_spec(spec: &str) -> Self {
    if let Some(cap) = SPEC_RE.captures(spec) {
      MuDimension(fixpoint(cap.get(1).map_or("", |m| m.as_str()).parse::<f32>().unwrap(), Some(UNITY as f32)) as f32)
    } else {
      MuDimension(kround(spec.parse::<f32>().unwrap()) as f32)
    }
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