use std::fmt;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::common::error::*;
use crate::common::numeric_ops::{NumericOps, UNITY, UNITY_F64, fixpoint, kround, round_to};
use crate::common::object::Object;
use crate::definition::register::RegisterType;
use crate::state::*;
use crate::tokens::Tokens;
use crate::{Digested, RegisterValue};

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
  pub fn spec_to_f64(spec: &str) -> Result<f64> {
    if spec.is_empty() {
      Ok(0.0)
    } else if let Some(cap) = SPEC_RE.captures(spec) {
      // Dimensions given.
      let num_str = cap.get(1).map_or(String::new(), |m| m.as_str().to_string());
      let num: f64 = num_str.parse::<f64>().expect(&num_str);
      let unit = cap.get(2).map_or(String::new(), |m| m.as_str().to_string());
      let converted_unit = convert_unit(&unit);
      Ok(fixpoint(num, Some(converted_unit)) as f64)
    } else {
      // When scaled points passed in (typically the result of Perl calculations on other
      // Dimensions), you might think truncation (int) is more TeX-like.
      // Recall that TeX arithmatic truncates, whereas reading by Gullet tends to round.
      // The Perl arithmetic is replacing an unknown combination of those truncates & rounds.
      // As it turns out, using int() here results in non-terminating loops in pgf/tikz.
      // So, we use round (Knuth style)
      // Note that divide and such explicitly use int(), however!
      Ok(kround(spec.parse::<f64>().expect(spec)) as f64)
    }
  }
}

impl std::str::FromStr for Dimension {
  type Err = crate::common::Error;
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
