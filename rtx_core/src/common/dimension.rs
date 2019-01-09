use crate::common::number::Number;
use crate::definition::register::NumericOps;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dimension {
  number: f32,
}

impl NumericOps for Dimension {
  fn value_of(self) -> f32 { self.number }
  fn new<T: Into<f32>>(number: T) -> Self { Dimension { number: number.into() } }
}

impl Dimension {
  /// Utility for formatting scaled points sanely.
  fn point_format(self) -> String {
    // As much as I'd like to make this more friendly & readable
    // there's TeX code that depends on getting enough precision
    // If you use %.5f, tikz (for example) will sometimes hang trying to do arithmetic!
    // But see toAttribute for friendlier forms....
    // [do we need the juggling in attributeFormat to be reproducible?]

    let mut s = s!("{:.6}", self.number / 65536.0);
    if s.contains('.') {
      s = s.trim_end_matches('0').to_string();
    }
    if s.ends_with('.') {
      s += "0"; // Seems TeX prints .0 which in odd corner cases, people use?
    }
    s!("{}pt", s)
  }

  fn attribute_format(self) -> String { s!("{:.1}pt", Number::round_to(self.value_of() / 65536.0, Some(1))) }

  pub fn to_string(self) -> String { self.point_format() }

  pub fn to_attribute(self) -> String { self.attribute_format() }
}
