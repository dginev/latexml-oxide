use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;

use super::dimension::fixedformat;
use crate::common::numeric_ops::{fixpoint, kround, NumericOps, UNITY_F64};
use crate::definition::register::RegisterType;
use crate::{Object};

static MUDIM_SPEC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(-?\d*\.?\d*)mu$").unwrap());

#[derive(Debug, Copy, Clone, PartialEq, Default, Eq)]
pub struct MuDimension(pub i64);

impl NumericOps for MuDimension {
  fn new(number: i64) -> Self { MuDimension(number) }
  fn new_f64(number: f64) -> Self { MuDimension(kround(number)) }
  fn value_of(self) -> i64 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::MuDimension }
  fn unit(&self) -> Option<&'static str> { Some("mu") }
}

impl fmt::Display for MuDimension {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", fixedformat(self.0, self.unit()))
  }
}
impl Object for MuDimension {}

impl MuDimension {
  pub fn new_spec(spec: &str) -> Self {
    if let Some(cap) = MUDIM_SPEC_RE.captures(spec) {
      MuDimension(fixpoint(
        cap
          .get(1)
          .map_or("", |m| m.as_str())
          .parse::<f64>()
          .unwrap(),
        Some(UNITY_F64),
      ))
    } else {
      MuDimension(kround(spec.parse::<f64>().unwrap()))
    }
  }
}
