///! Token List constructors.
use common::error::*;
use state::State;
use token::*;
use stomach::{Stomach};
use Digested;

// Form a Tokens list of Token's
// Flatten the arguments Token's and Tokens's into plain Token's
// .... Efficiently! since this seems to be called MANY times.
#[derive(Debug, Clone)]
pub struct Tokens {
  pub tokens: Vec<Token>
}
impl Default for Tokens {
  fn default() -> Self {
    Tokens {
      tokens: Vec::new()
    }
  }
}

#[macro_export]
macro_rules! Tokens(($( $tokens:expr ),*) => ({
  use $crate::tokens::Tokens;
  Tokens { tokens: vec![$($tokens)*] }
}));

impl Tokens {
  pub fn new(tokens : Vec<Token>) -> Self {
    Tokens { tokens: tokens }
  }

  /// Return a list of the tokens making up this Tokens
  pub fn unlist(self) -> Vec<Token> {
    self.tokens
  }

  /// Checks if there are tokens present
  pub fn is_empty(&self) -> bool {
    self.tokens.is_empty()
  }

  /// Return a shallow copy of the Tokens
  pub fn clone(&self) -> Self {
    Tokens { tokens: self.tokens.clone() }
  }

  /// Return a string containing the TeX form of the Tokens
  pub fn revert(self) -> Vec<Token> {
    self.tokens
  }

  /// toString is used often, and for more keyword-like reasons,
  /// NOT for creating valid TeX (use revert or UnTeX for that!)
  pub fn to_string(&self) -> String {
    self.tokens.iter().map(|t| t.text.as_str()).collect::<Vec<_>>().join("")
  }

  /// Methods for overloaded ops.
  pub fn equals(&self, other: Tokens) -> bool {
    let self_tokens = &self.tokens;
    let other_tokens = &other.tokens;
    if self_tokens.len() != other_tokens.len() {
      false
    } else {
      for it in self_tokens.iter().zip(other_tokens.iter()) {
        let (self_token, other_token) = it;
        if self_token != other_token {
          return false;
        }
      }
      true
    }
  }

  pub fn stringify(self) -> String {
    "Tokens[".to_string() +
      &self.tokens.iter().map(|t| t.to_string())
        .collect::<Vec<_>>().join(",")
    + "]"
  }

  pub fn be_digested(self, stomach : &mut Stomach, state: &mut State) -> Result<Digested> {
    stomach.digest(self, state)
  }

  pub fn neutralize(self, extraspecials: &Vec<Token>, state: &State) -> Tokens {
    Tokens {
      tokens: self.tokens.into_iter().map(|t| t.neutralize(extraspecials, state) ).collect::<Vec<_>>()
    }
  }

  pub fn is_balanced(&self) -> bool {
    let mut level = 0;
    for t in &self.tokens {
      level += match t.code {
        Catcode::BEGIN => 1,
        Catcode::END => -1,
        _ => 0
      };
    }
    level == 0
  }
}
