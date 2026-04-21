use std::fmt;

use crate::Digested;
use crate::common::error::Result;
use crate::common::float::Float;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::token::Catcode;
use crate::token::Token;
use crate::tokens::Tokens;

/// A pair of numerical values, typically (x,y) coordinates.
/// Perl: LaTeXML::Core::Pair
#[derive(Debug, Clone, PartialEq)]
pub struct Pair {
  pub x: Float,
  pub y: Float,
}

impl Default for Pair {
  fn default() -> Self { Pair { x: Float(0.0), y: Float(0.0) } }
}

impl Pair {
  pub fn new(x: Float, y: Float) -> Self { Pair { x, y } }

  pub fn get_x(&self) -> Float { self.x }
  pub fn get_y(&self) -> Float { self.y }

  pub fn to_attribute(&self) -> String {
    format!("{},{}", self.x.to_attribute(), self.y.to_attribute())
  }
}

impl Object for Pair {
  fn revert(&self) -> Result<Tokens> {
    let mut toks: Vec<Token> = vec![Token::new("(", Catcode::OTHER)];
    toks.extend(ExplodeText!(&self.x.to_string()));
    toks.push(Token::new(",", Catcode::OTHER));
    toks.extend(ExplodeText!(&self.y.to_string()));
    toks.push(Token::new(")", Catcode::OTHER));
    Ok(Tokens::new(toks))
  }

  fn be_digested(self) -> Result<Digested> {
    // Preserve pair values through digestion via RegisterValue::Pair
    Ok(Digested::from(crate::RegisterValue::Pair(self)))
  }
}

impl fmt::Display for Pair {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "({},{})", self.x, self.y) }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pair_new_and_getters() {
    let p = Pair::new(Float(1.5), Float(-2.5));
    assert_eq!(p.get_x().0, 1.5);
    assert_eq!(p.get_y().0, -2.5);
  }

  #[test]
  fn pair_default_is_origin() {
    let p = Pair::default();
    assert_eq!(p.get_x().0, 0.0);
    assert_eq!(p.get_y().0, 0.0);
  }

  #[test]
  fn pair_display_format() {
    let p = Pair::new(Float(1.0), Float(2.0));
    assert_eq!(format!("{p}"), "(1.0,2.0)");
  }

  #[test]
  fn pair_display_negative() {
    let p = Pair::new(Float(-1.5), Float(-2.5));
    assert_eq!(format!("{p}"), "(-1.5,-2.5)");
  }

  #[test]
  fn pair_to_attribute_uses_float_attribute_format() {
    // to_attribute delegates to Float::to_attribute for each axis.
    // Float's NumericOps::to_attribute default is to_string(), so
    // output matches Display.
    let p = Pair::new(Float(3.0), Float(4.0));
    assert_eq!(p.to_attribute(), "3.0,4.0");
  }

  #[test]
  fn pair_equality() {
    let a = Pair::new(Float(1.0), Float(2.0));
    let b = Pair::new(Float(1.0), Float(2.0));
    let c = Pair::new(Float(1.0), Float(3.0));
    assert_eq!(a, b);
    assert_ne!(a, c);
  }
}
