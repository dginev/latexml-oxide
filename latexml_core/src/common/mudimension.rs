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
  // XML attribute output is pt-typed by convention. Convert mu→pt via
  // Perl `MuGlue::ptValue` two-step truncation so XMHint width attrs
  // emit `1.66663pt` not `3.0mu` (and downstream lpadding/rpadding
  // transferred from the XMHint width keep the pt unit).
  fn to_attribute(&self) -> String {
    let fs = crate::state::lookup_font()
      .and_then(|f| f.get_size())
      .unwrap_or(10.0);
    let muwidth = (fs * UNITY_F64 / 18.0) as i64;
    let pt_scaled = ((self.0 as f64 * muwidth as f64 / UNITY_F64).trunc()) as i64;
    super::dimension::attribute_format(pt_scaled, Some("pt"))
  }
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
      let num: f64 = cap
        .get(1)
        .map_or("", |m| m.as_str())
        .parse::<f64>()
        .unwrap_or(0.0);
      MuDimension(fixpoint(num, Some(UNITY_F64)))
    } else {
      // Perl parity: bad input coerces to 0.
      MuDimension(kround(spec.parse::<f64>().unwrap_or(0.0)))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mudim_default_is_zero() {
    assert_eq!(MuDimension::default().value_of(), 0);
  }

  #[test]
  fn mudim_new_builds_value() {
    assert_eq!(MuDimension::new(65536).value_of(), 65536);
    assert_eq!(MuDimension::new(0).value_of(), 0);
    assert_eq!(MuDimension::new(-100).value_of(), -100);
  }

  #[test]
  fn mudim_new_f64_rounds_knuthian() {
    assert_eq!(MuDimension::new_f64(0.0).value_of(), 0);
    assert_eq!(MuDimension::new_f64(1.0).value_of(), 1);
    assert_eq!(MuDimension::new_f64(-1.0).value_of(), -1);
  }

  #[test]
  fn mudim_register_type_is_mudimension() {
    assert_eq!(
      MuDimension::default().register_type(),
      RegisterType::MuDimension
    );
  }

  #[test]
  fn mudim_unit_is_mu() {
    assert_eq!(MuDimension::default().unit(), Some("mu"));
  }

  #[test]
  fn mudim_display_includes_mu_unit() {
    let m = MuDimension::new(65536); // 1mu in scaled units
    let out = format!("{m}");
    assert!(out.ends_with("mu"), "got {out:?}");
  }

  #[test]
  fn new_spec_parses_numeric_with_mu() {
    // "1mu" parses as MuDimension(fixpoint(1.0, UNITY_F64)).
    let m = MuDimension::new_spec("1mu");
    assert_ne!(m.value_of(), 0, "1mu should not be zero");
    // "0mu" is zero.
    let m0 = MuDimension::new_spec("0mu");
    assert_eq!(m0.value_of(), 0);
  }

  #[test]
  fn new_spec_empty_numeric_part_is_zero() {
    // "mu" (bare unit, no number) coerces to 0 via Perl-parity.
    let m = MuDimension::new_spec("mu");
    assert_eq!(m.value_of(), 0);
  }

  #[test]
  fn new_spec_bad_input_is_zero() {
    // Non-matching spec falls through to bare-number parse → 0 if that
    // also fails.
    let m = MuDimension::new_spec("not a number");
    assert_eq!(m.value_of(), 0);
  }

  #[test]
  fn mudim_equality() {
    assert_eq!(MuDimension::new(100), MuDimension::new(100));
    assert_ne!(MuDimension::new(100), MuDimension::new(101));
  }
}
