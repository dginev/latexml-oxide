use crate::definition::register;
use crate::definition::register::{NumericOps, RegisterType};
use std::fmt;
use crate::common::number::kround;

pub static UNITY : usize = 65536;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dimension(pub f32);

impl NumericOps for Dimension {
  fn value_of(self) -> f32 { self.0 }
  fn register_type(&self) -> RegisterType { RegisterType::Dimension }
  fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() + other.value_of())
  }
  fn subtract<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() - other.value_of())
  }
}

impl Default for Dimension {
  fn default() -> Self { Dimension(0.0) }
}

impl fmt::Display for Dimension {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", fixedformat(self.0 as i32, Some("pt")))
  }
}

impl Dimension {
  pub fn new(number: f32) -> Self { Dimension(kround(number) as f32) }

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

  /// Utility for formatting scaled points sanely.
  pub fn point_format(num: f32) -> String {
    // As much as I'd like to make this more friendly & readable
    // there's TeX code that depends on getting enough precision
    // If you use %.5f, tikz (for example) will sometimes hang trying to do arithmetic!
    // But see toAttribute for friendlier forms....
    // [do we need the juggling in attributeFormat to be reproducible?]

    let mut s = s!("{:.5}", num / 65536.0);
    if s.contains('.') {
      s = s.trim_end_matches('0').to_string();
    }
    if s.ends_with('.') {
      s += "0"; // Seems TeX prints .0 which in odd corner cases, people use?
    }
    s!("{}pt", s)
  }

  pub fn to_attribute(self) -> String { attribute_format(self.value_of(), Some("pt")) }
}
// Dimension!() macro is in setup.rs, since it binds state

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

// This is Knuth's print_scaled (See TeX the Program, \S 103)
// It (should) round-trip with kround.
pub fn fixedformat(mut s:i32, unit_opt: Option<&str>) -> String {
  let mut string = String::new();
  if s < 0 {
    string.push('-');
    s = -s;
  }
  string.push_str(&(((s as f32 / UNITY as f32).trunc() as i32).to_string()));
  string.push('.');
  s = 10 * (s % UNITY as i32) + 5;
  let mut delta = 10i32;
  loop {
    if delta > UNITY as i32 {
      s += 0x8000 - 50000;
    }
    string.push_str(&((s as f32 / UNITY as f32).trunc() as i32).to_string());
    s = 10 * (s % UNITY as i32);
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

pub fn attribute_format(sp: f32, unit_opt: Option<&str>) -> String {
  let unit = unit_opt.unwrap_or("pt");
  s!("{:.1}{}", register::round_to(sp / UNITY as f32, Some(1)), unit)
}