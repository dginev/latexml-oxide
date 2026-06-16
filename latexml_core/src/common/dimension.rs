use std::fmt;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
  Digested, RegisterValue,
  common::{
    error::*,
    numeric_ops::{NumericOps, UNITY, UNITY_F64, fixpoint_unit, kround, round_to},
    object::Object,
  },
  definition::register::RegisterType,
  state::*,
  tokens::Tokens,
};

static SPEC_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(-?\d*\.?\d*)([a-zA-Z][a-zA-Z])$").unwrap());

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Dimension(pub i64);

impl Object for Dimension {
  fn revert(&self) -> Result<Tokens> { Ok(Tokens::new(ExplodeText!(&self.to_string()))) }
  fn be_digested(self) -> Result<Digested>
  where
    Self: Sized,
    Self: fmt::Debug,
  {
    Ok(Digested::from(RegisterValue::Dimension(self)))
  }
}
impl NumericOps for Dimension {
  fn new(number: i64) -> Self { Dimension(number) }
  fn new_f64(number: f64) -> Self { Dimension(kround(number)) }
  fn value_of(self) -> i64 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::Dimension }
  fn unit(&self) -> Option<&'static str> { Some("pt") }
  fn to_attribute(&self) -> String { attribute_format(self.value_of(), self.unit()) }
}

impl fmt::Display for Dimension {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", fixedformat(self.0, self.unit()))
  }
}

impl Dimension {
  /// Convert dimension to em units, using the given font's EM width (or current font if None).
  pub fn em_value(&self, prec: Option<u8>, font: Option<&crate::common::font::Font>) -> f64 {
    let em_width: f64 = if let Some(f) = font {
      f.get_em_width() as f64
    } else {
      lookup_font()
        .map(|f| f.get_em_width() as f64)
        .unwrap_or(UNITY_F64 * 10.0) // fallback: 10pt
    };
    round_to(self.0 as f64 / em_width, prec)
  }

  pub fn spec_to_f64(spec: &str) -> Result<f64> {
    if spec.is_empty() {
      Ok(0.0)
    } else if let Some(cap) = SPEC_RE.captures(spec) {
      // Dimensions given. SPEC_RE's numeric part is `-?\d*\.?\d*` which
      // can match empty (e.g. input "pt"); Perl's `fixpoint($1, ...)`
      // coerces "" → 0 via numeric context, so unwrap_or(0.0) keeps parity.
      let num_str = cap.get(1).map_or("", |m| m.as_str());
      let num: f64 = num_str.parse::<f64>().unwrap_or(0.0);
      let unit = cap.get(2).map_or("", |m| m.as_str());
      let (conv_num, conv_den) = convert_unit_ratio(unit);
      Ok(fixpoint_unit(num, conv_num, conv_den) as f64)
    } else {
      // When scaled points passed in (typically the result of Perl calculations on other
      // Dimensions), you might think truncation (int) is more TeX-like.
      // Recall that TeX arithmatic truncates, whereas reading by Gullet tends to round.
      // The Perl arithmetic is replacing an unknown combination of those truncates & rounds.
      // As it turns out, using int() here results in non-terminating loops in pgf/tikz.
      // So, we use round (Knuth style)
      // Note that divide and such explicitly use int(), however!
      // Perl parity: `kround($spec || 0)` — non-numeric input coerces to 0.
      Ok(kround(spec.parse::<f64>().unwrap_or(0.0)) as f64)
    }
  }
}

impl std::str::FromStr for Dimension {
  type Err = Error;
  fn from_str(spec: &str) -> Result<Dimension> {
    Ok(Dimension::new_f64(Dimension::spec_to_f64(spec)?))
  }
}

// Dimension!() macro is in setup.rs, since it binds state

// This is Knuth's print_scaled (See TeX the Program, \S 103)
// It (should) round-trip with kround.
pub fn fixedformat(mut s: i64, unit_opt: Option<&str>) -> String {
  let mut string = String::new();
  if s < 0 {
    string.push('-');
    s = -s;
  }
  string.push_str(&(s / UNITY).to_string());
  string.push('.');
  s = 10 * (s % UNITY) + 5;
  let mut delta = 10;
  loop {
    if delta > UNITY {
      s += 0x8000 - 50000;
    }
    string.push_str(&(s / UNITY).to_string());
    s = 10 * (s % UNITY);
    delta *= 10;
    if s <= delta {
      break;
    }
  }
  if let Some(unit) = unit_opt {
    string.push_str(unit);
  }
  string
}

pub fn attribute_format(sp: i64, unit_opt: Option<&str>) -> String {
  let unit = unit_opt.unwrap_or("pt");
  s!("{:.1}{unit}", round_to(sp as f64 / UNITY_F64, Some(1)))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fixedformat_zero() {
    // Knuth's print_scaled of 0 with a unit.
    assert_eq!(fixedformat(0, Some("pt")), "0.0pt");
    assert_eq!(fixedformat(0, None), "0.0");
  }

  #[test]
  fn fixedformat_one_sp_unit() {
    // UNITY = 65536 scaled-points = 1pt.
    assert_eq!(fixedformat(UNITY, Some("pt")), "1.0pt");
    assert_eq!(fixedformat(2 * UNITY, Some("pt")), "2.0pt");
  }

  #[test]
  fn fixedformat_negative() {
    assert_eq!(fixedformat(-UNITY, Some("pt")), "-1.0pt");
    assert_eq!(fixedformat(-2 * UNITY, Some("pt")), "-2.0pt");
  }

  #[test]
  fn fixedformat_half_pt() {
    // 0.5 pt = UNITY/2 = 32768 sp.
    let out = fixedformat(UNITY / 2, Some("pt"));
    assert_eq!(out, "0.5pt");
  }

  #[test]
  fn attribute_format_defaults_to_pt() {
    assert_eq!(attribute_format(UNITY, None), "1.0pt");
    assert_eq!(attribute_format(UNITY, Some("pt")), "1.0pt");
  }

  #[test]
  fn attribute_format_other_unit() {
    assert_eq!(attribute_format(UNITY, Some("in")), "1.0in");
  }

  #[test]
  fn attribute_format_rounds_to_one_decimal() {
    // sp = UNITY * 1.25 → 1.3pt (round to 1 decimal, half-up).
    let sp = (UNITY_F64 * 1.25) as i64;
    let out = attribute_format(sp, None);
    assert_eq!(out, "1.3pt", "got {out:?}");
  }

  #[test]
  fn spec_to_f64_empty_is_zero() {
    assert_eq!(Dimension::spec_to_f64("").unwrap(), 0.0);
  }

  #[test]
  fn spec_to_f64_bare_number_is_scaled() {
    // No unit → treated as scaled-points directly; the value is
    // kround()-ed through, so "65536" stays 65536.
    let out = Dimension::spec_to_f64("65536").unwrap();
    assert_eq!(out, 65536.0);
  }
}
