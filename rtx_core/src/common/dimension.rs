use std::fmt;
use crate::definition::register;
use crate::definition::register::{NumericOps, RegisterType};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dimension(pub f32);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MuDimension(pub f32);

impl NumericOps for Dimension {
  fn value_of(self) -> f32 { self.0 }
  fn new<T: Into<f32>>(number: T) -> Self { Dimension(number.into()) }
  fn register_type(&self) -> RegisterType { RegisterType::Dimension }
}

impl NumericOps for MuDimension {
  fn value_of(self) -> f32 { self.0 }
  fn new<T: Into<f32>>(number: T) -> Self { MuDimension(number.into()) }
  fn register_type(&self) -> RegisterType { RegisterType::MuDimension }
}

impl Default for Dimension {
  fn default() -> Self { Dimension(0.0) }
}
impl Default for MuDimension {
  fn default() -> Self { MuDimension(0.0) }
}

impl fmt::Display for Dimension {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", Dimension::point_format(self.0))
  }
}
impl fmt::Display for MuDimension {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", Dimension::point_format(self.0))
  }
}


impl Dimension {
  /// Utility for formatting scaled points sanely.
  pub fn point_format(num: f32) -> String {
    // As much as I'd like to make this more friendly & readable
    // there's TeX code that depends on getting enough precision
    // If you use %.5f, tikz (for example) will sometimes hang trying to do arithmetic!
    // But see toAttribute for friendlier forms....
    // [do we need the juggling in attributeFormat to be reproducible?]

    let mut s = s!("{:.6}", num / 65536.0);
    if s.contains('.') {
      s = s.trim_end_matches('0').to_string();
    }
    if s.ends_with('.') {
      s += "0"; // Seems TeX prints .0 which in odd corner cases, people use?
    }
    s!("{}pt", s)
  }

  fn attribute_format(self) -> String { s!("{:.1}pt", register::round_to(self.value_of() / 65536.0, Some(1))) }

  pub fn to_attribute(self) -> String { self.attribute_format() }
}

// Dimension!() macro is in setup.rs, since it binds state
