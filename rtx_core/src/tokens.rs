///! Token List constructors.
use crate::fmt;
use log::*;
use proc_macro2::{Ident, Punct, Span, Spacing};
use quote::{ToTokens, TokenStreamExt, quote};

use std::collections::VecDeque;
use std::fmt::Display;

use crate::common::error::*;
use crate::common::number::Number;
use crate::common::dimension::{MuDimension,Dimension};
use crate::common::glue::{Glue, MuGlue};
use crate::definition::register::RegisterValue;
use crate::state::State;
use crate::stomach::Stomach;
use crate::token::*;
use crate::Digested;

const UNTEX_LINELENGTH: usize = 78;

#[derive(Debug, Clone, PartialEq)]
pub struct Tokens(pub Vec<Token>);

impl Default for Tokens {
  fn default() -> Self { Tokens(Vec::new()) }
}

#[macro_export]
macro_rules! Tokens(
  ($( $tokens:expr ),*) => ({
    let mut collected : Vec<Token> = Vec::new();
    $(
      let t_vec : Vec<Token> = $tokens.into();
      collected.extend(t_vec);
    );*;
    $crate::tokens::Tokens::new(collected)
  }));
// We also need convenient auxiliaries, including auto-casting
impl From<Vec<Token>> for Tokens {
  fn from(ts: Vec<Token>) -> Tokens { Tokens::new(ts) }
}

impl From<Token> for Tokens {
  fn from(t: Token) -> Tokens { Tokens::new(vec![t]) }
}
impl From<Tokens> for Result<Tokens> {
  fn from(t: Tokens) -> Result<Tokens> { Ok(t) }
}
impl From<Token> for Result<Tokens> {
  fn from(t: Token) -> Result<Tokens> { Ok(t.into()) }
}
impl From<Token> for Vec<Token> {
  fn from(t: Token) -> Vec<Token> { vec![t] }
}

impl From<Tokens> for Token {
  fn from(ts: Tokens) -> Token { (&ts).into() }
}

impl<'a> From<&'a Tokens> for Token {
  fn from(ts: &'a Tokens) -> Token {
    if ts.is_stub() {
      Token::default()
    } else if ts.0.len() == 1 {
      ts.0.first().unwrap().clone()
    } else {
      warn!(target: "expected:token", "multiple Tokens cast into a single Token");
      ts.0.first().unwrap().clone()
    }
  }
}

impl Display for Tokens {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for t in &self.0 {
      write!(f, "{}", t)?;
    }
    Ok(())
  }
}

impl Tokens {
  pub fn new(tokens: Vec<Token>) -> Self { Tokens(tokens) }

  /// Return a list of the tokens making up this Tokens
  pub fn unlist(self) -> Vec<Token> { self.0 }

  /// Are there any tokens at all contained in this Tokens object
  pub fn is_empty(&self) -> bool { self.0.is_empty() }

  /// Are there any non-stub tokens contained in this Tokens object?
  pub fn is_stub(&self) -> bool { self.is_empty() || self.0.iter().all(|t| *t == *MOCK_TOKEN) }

  /// Number of contained Token entries
  pub fn len(&self) -> usize { self.0.len() }

  /// Return a string containing the TeX form of the Tokens
  pub fn revert(self) -> Vec<Token> { self.0 }

  /// toString is used often, and for more keyword-like reasons,
  /// NOT for creating valid TeX (use revert or UnTeX for that!)
  pub fn to_string(&self) -> String {
    let mut result = String::new();
    for t in self.0.iter() {
      result.push_str(&t.text);
    }
    result
  }

  /// to_number casts back to a parsed Number (usually via gullet.read_number)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_number(&self) -> Number {
    let token: Token = self.into();
    token.to_number()
  }

  /// to_dimension casts back to a parsed Dimension (usually via gullet.read_dimension)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_dimension(&self) -> Dimension {
    let token: Token = self.into();
    token.to_dimension()
  }

  /// to_glue casts back to a parsed Glue (usually via gullet.read_glue)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_glue(&self) -> Glue {
    let token: Token = self.into();
    token.to_glue()
  }

  /// to_mu_glue casts back to a parsed MuGlue (usually via gullet.read_mu_glue)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_mu_glue(&self) -> MuGlue {
    let token: Token = self.into();
    token.to_mu_glue()
  }

  /// to_mu_dimension casts back to a parsed MuGlue (usually via gullet.read_mu_glue)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_mu_dimension(&self) -> MuDimension {
    let token: Token = self.into();
    token.to_mu_dimension()
  }

  /// Methods for overloaded ops.
  pub fn equals(&self, other: Tokens) -> bool {
    let self_tokens = &self.0;
    let other_tokens = &other.0;
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

  pub fn stringify(&self) -> String { s!("Tokens[{}]", &self.0.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(",")) }

  pub fn value_of(&self, args: Vec<Token>, state: &mut State) -> Option<RegisterValue> {
    let token: &Token = &self.0[0];
    token.value_of(args, state)
  }

  pub fn be_digested(self, stomach: &mut Stomach, state: &mut State) -> Result<Digested> { stomach.digest(self, state) }

  pub fn neutralize(self, extraspecials: &[Token], state: &State) -> Tokens {
    Tokens(self.0.into_iter().map(|t| t.neutralize(extraspecials, state)).collect::<Vec<_>>())
  }

  pub fn is_balanced(&self) -> bool {
    let mut level = 0;
    for t in &self.0 {
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
  pub fn substitute_parameters(&self, args: Vec<Tokens>) -> Self {
    let mut result = Vec::new();
    let mut in_tokens = self.0.iter();
    while let Some(token) = in_tokens.next() {
      if token.code != Catcode::PARAM {
        // Non '#'; copy it
        result.push(token.clone());
      } else if let Some(token2) = in_tokens.next() {
        if token2.code != Catcode::PARAM {
          // Not multiple '#'; read arg.
          let arg_number = token2.text.parse::<usize>().unwrap();
          let arg = &args[arg_number - 1];
          result.extend(arg.clone().unlist());
        } else {
          // Duplicated '#', copy 2nd '#'
          result.push(token2.clone());
        }
      }
    }
    Tokens::new(result)
  }

  pub fn untex(&self, state: &mut State) -> String {
    let tokens = self.clone().revert();
    let mut tokens: VecDeque<Token> = tokens.into();
    let mut result = String::new();
    let mut length = 0;
    let mut level = 0;
    let mut prevs = String::new();
    let mut prevcc = Catcode::COMMENT;
    while !tokens.is_empty() {
      let token = tokens.pop_front().unwrap();
      let cc = token.get_catcode();
      if cc == Catcode::COMMENT {
        continue;
      }
      let mut s = token.get_string().to_owned();
      if cc == Catcode::LETTER {
        // keep "words" together, just for aesthetics
        while !tokens.is_empty() && tokens[0].get_catcode() == Catcode::LETTER {
          s.push_str(tokens.pop_front().unwrap().get_string());
        }
      }

      let l = s.len();
      if cc == Catcode::BEGIN {
        level += 1;
      }
      //  Seems a reasonable & safe time to line break, for readability, etc.
      if cc == Catcode::SPACE && s == "\n" {
        // preserve newlines already present
        if length > 0 {
          result = s;
          length = 0;
        }
      // If this token is a letter (or otherwise starts with a letter or digit): space or linebreak
      } else {
        let last_prevs = prevs.chars().last().unwrap_or('_');
        let prev_is_letter = if let Some(prevs_cc) = state.lookup_catcode(last_prevs) {
          prevs_cc == Catcode::LETTER
        } else {
          false
        };

        if (cc == Catcode::LETTER || (cc == Catcode::OTHER && s.chars().next().unwrap_or('_').is_alphanumeric()))
          && prevcc == Catcode::CS
          && prev_is_letter
        {
          // Insert a (virtual) space before a letter if previous token was a CS w/letters
          // This is required for letters, but just aesthetic for digits (to me?)
          // Of course, use a newline if we're already at end
          let space = if length > 0 && length + l > UNTEX_LINELENGTH { '\n' } else { ' ' };
          result.push(space);
          result.push_str(&s);
          length += 1 + l;
        } else if length > 0 && (length + l > UNTEX_LINELENGTH) && tokens.len() > 1 {
          // linebreak before this token? and not at end!
          // Or even within an arg!
          result.push_str("%\n");
          result.push_str(&s);
          length = l; // with %, so that it "disappears"
        } else {
          result.push_str(&s);
          length += l;
        }
        if cc == Catcode::END {
          level -= 1;
        }
        prevs = s;
        prevcc = cc;
      }
    }
    // Patch up nesting for valid TeX !!!
    if level > 0 {
      for _ in 0..level {
        result.push('}');
      }
    } else if level < 0 {
      for _ in 0..(-level) {
        result = String::from("{") + &result;
      }
    }
    result
  }
}

impl ToTokens for Tokens {
  fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
    let d = &self.0;
    stream.extend(quote! {
        Tokens(<[Token]>::into_vec(Box::new([ #(#d),* ])))
    });
  }
}

impl ToTokens for Catcode {
  fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
    use crate::token::Catcode::*;
    let kind = match *self {
      ESCAPE => "ESCAPE",
      BEGIN => "BEGIN",
      END => "END",
      MATH => "MATH",
      ALIGN => "ALIGN",
      EOL => "EOL",
      PARAM => "PARAM",
      SUPER => "SUPER",
      SUB => "SUB",
      SPACE => "SPACE",
      NOTEXPANDED => "NOTEXPANDED",
      // Non-primitive
      IGNORE => "IGNORE",
      LETTER => "LETTER",
      OTHER => "OTHER",
      ACTIVE => "ACTIVE",
      COMMENT => "COMMENT",
      INVALID => "INVALID",
      CS => "CS",
      MARKER => "MARKER",
    };
    stream.append(Ident::new("Catcode", Span::call_site()));
    stream.append(Punct::new(':', Spacing::Joint));
    stream.append(Punct::new(':', Spacing::Alone));
    stream.append(Ident::new(kind, Span::call_site()));
  }
}

impl ToTokens for Token {
  fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
    let text = &self.text;
    let code = &self.code;
    stream.extend(quote! {
      Token {
        text: Cow::Borrowed(#text),
        code: #code
      }
    });
  }
}
