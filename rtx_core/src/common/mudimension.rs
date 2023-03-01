use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::fmt;

use super::dimension::fixedformat;
use crate::common::numeric_ops::{fixpoint, kround, NumericOps, UNITY_F32};
use crate::definition::register::RegisterType;
use crate::{Locator, Object};

lazy_static! {
  static ref MUDIM_SPEC_RE: Regex = Regex::new(r"^(-?\d*\.?\d*)mu$").unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq, Default, Eq)]
pub struct MuDimension(pub i64);

impl NumericOps for MuDimension {
  fn new(number: i64) -> Self { MuDimension(number) }
  fn new_f32(number: f32) -> Self { MuDimension(kround(number)) }
  fn value_of(self) -> i64 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::MuDimension }
  fn unit(&self) -> Option<&'static str> { Some("mu") }
}

impl fmt::Display for MuDimension {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", fixedformat(self.0, self.unit())) }
}
impl Object for MuDimension {
  fn get_locator(&self) -> Option<Cow<Locator>> { None }
}

impl MuDimension {
  pub fn new_spec(spec: &str) -> Self {
    if let Some(cap) = MUDIM_SPEC_RE.captures(spec) {
      MuDimension(fixpoint(
        cap.get(1).map_or("", |m| m.as_str()).parse::<f32>().unwrap(),
        Some(UNITY_F32),
      ))
    } else {
      MuDimension(kround(spec.parse::<f32>().unwrap()))
    }
  }
}
