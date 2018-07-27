///! Token List constructors.
use fmt;
use std::fmt::Display;

use common::error::*;
use quote::ToTokens;
use quote::Tokens as QTokens;
use state::State;
use stomach::Stomach;
use token::*;
use Digested;

// Form a Tokens list of Token's
// Flatten the arguments Token's and Tokens's into plain Token's
// .... Efficiently! since this seems to be called MANY times.
#[derive(Debug, Clone, PartialEq)]
pub struct Tokens {
  pub tokens: Vec<Token>,
}
impl Default for Tokens {
  fn default() -> Self { Tokens { tokens: Vec::new() } }
}
impl ToTokens for Tokens {
  fn to_tokens(&self, tokens: &mut QTokens) {
    tokens.append("Tokens {tokens: vec!");
    self.tokens.to_tokens(tokens);
    tokens.append("}");
  }
}

#[macro_export]
macro_rules! Tokens(
  ($( $tokens:expr ),*) => ($crate::tokens::Tokens{ tokens: vec![$($tokens),*] });
);

impl Display for Tokens {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for t in &self.tokens {
      write!(f, "{}", t)?;
    }
    Ok(())
  }
}

impl Tokens {
  pub fn new(tokens: Vec<Token>) -> Self { Tokens { tokens } }

  /// Return a list of the tokens making up this Tokens
  pub fn unlist(self) -> Vec<Token> { self.tokens }

  /// Checks if there are tokens present
  pub fn is_empty(&self) -> bool { self.tokens.is_empty() }

  /// Return a string containing the TeX form of the Tokens
  pub fn revert(self) -> Vec<Token> { self.tokens }

  /// toString is used often, and for more keyword-like reasons,
  /// NOT for creating valid TeX (use revert or UnTeX for that!)
  pub fn to_string(&self) -> String {
    self
      .tokens
      .iter()
      .map(|t| t.text.as_str())
      .collect::<Vec<_>>()
      .join("")
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

  // stopgap, how do we unpack! gullet-stage arguments without the unwrap?
  // should we unify the interfaces so that Options are always used? Could be cumbursome...
  pub fn unwrap_or_default(self) -> Tokens { self }

  pub fn stringify(self) -> String {
    s!(
      "Tokens[{}]",
      &self
        .tokens
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(",")
    )
  }

  pub fn be_digested(self, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
    stomach.digest(self, state)
  }

  pub fn neutralize(self, extraspecials: &[Token], state: &State) -> Tokens {
    Tokens {
      tokens: self
        .tokens
        .into_iter()
        .map(|t| t.neutralize(extraspecials, state))
        .collect::<Vec<_>>(),
    }
  }

  pub fn is_balanced(&self) -> bool {
    let mut level = 0;
    for t in &self.tokens {
      level += match t.code {
        Catcode::BEGIN => 1,
        Catcode::END => -1,
        _ => 0,
      };
    }
    level == 0
  }

  // NOTE: Assumes each arg either undef or also Tokens
  // Using inline accessors on those assumptions
  pub fn substitute_parameters(self, args: Vec<Tokens>) -> Self {
    let mut result = Vec::new();
    let mut in_tokens = self.tokens.into_iter();
    while let Some(token) = in_tokens.next() {
      if token.code != Catcode::PARAM {
        // Non '#'; copy it
        result.push(token);
      } else if let Some(token2) = in_tokens.next() {
        if token2.code != Catcode::PARAM {
          // Not multiple '#'; read arg.
          let arg_number = token2.text.parse::<usize>().unwrap();
          let arg = &args[arg_number - 1];
          result.extend(arg.clone().unlist());
        } else {
          // Duplicated '#', copy 2nd '#'
          result.push(token2);
        }
      }
    }
    Tokens::new(result)
  }
}
