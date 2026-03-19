use std::fmt;

use crate::common::error::Result;
use crate::common::float::Float;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::token::Token;
use crate::tokens::Tokens;
use crate::token::Catcode;
use crate::Digested;

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
    // Pairs are typically used as parameter values, not digested
    Ok(Digested::from(crate::RegisterValue::Dimension(
      crate::common::dimension::Dimension(0),
    )))
  }
}

impl fmt::Display for Pair {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({},{})", self.x, self.y)
  }
}
