use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;

use super::dimension::fixedformat;
use crate::Object;
use crate::common::numeric_ops::{NumericOps, UNITY_F64, fixpoint, kround};
use crate::definition::register::RegisterType;

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
      // The numeric capture `-?\d*\.?\d*` can match empty (e.g. input
      // "mu"); Perl's fixpoint coerces "" → 0 via numeric context, so
      // unwrap_or(0.0) keeps parity.
      let num: f64 = cap.get(1).map_or("", |m| m.as_str()).parse::<f64>().unwrap_or(0.0);
      MuDimension(fixpoint(num, Some(UNITY_F64)))
    } else {
      // Perl parity: bad input coerces to 0.
      MuDimension(kround(spec.parse::<f64>().unwrap_or(0.0)))
    }
  }
}
