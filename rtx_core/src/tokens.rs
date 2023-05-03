//! Token List constructors.
use crate::fmt;
use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

use std::borrow::Cow;
use std::collections::VecDeque;
use std::convert::AsRef;
use std::fmt::Display;
use std::string::ToString;
use std::rc::Rc;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::float::Float;
use crate::common::glue::Glue;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::common::store::Stored;
use crate::keyvals::KeyVals;
use crate::state::State;
use crate::stomach::Stomach;
use crate::token::*;
use crate::Digested;

/// If untex is requested to add line-breaks, this is the line length it will allow
pub const UNTEX_LINELENGTH: usize = 78;
/// Use this to avoid reallocating a new empty Vec each time you need a placeholder Tokens return
/// value
pub const NO_TOKENS: Tokens = Tokens(Vec::new());
pub const NO_BORROWED_TOKENS : &Tokens = &NO_TOKENS;
/// Tokens are a thin wrapper over a vector of Token objects
/// usually read from a `Mouth`.
/// They are usually treated as an immutable interface, an have to be consumed via `.unlist()`
/// for access to the underlying data.
#[derive(Debug, Clone, Default)]
pub struct Tokens(Vec<Token>);

impl PartialEq for Tokens {
  fn eq(&self, other: &Tokens) -> bool {
    self.0.len() == other.0.len() && self.0.iter().zip(other.0.iter()).all(|(a, b)| a == b)
  }
}

/// convenience macro for assembling a Tokens object from different pieces (`Token`, `Vec<Token>`,
/// `Tokens`)
#[macro_export]
macro_rules! Tokens(
  () => ( $crate::tokens::NO_TOKENS );
  ($( $tokens:expr ),+) => ({
    let mut collected : Vec<$crate::token::Token> = Vec::new();
    $(
      let t_vec : Vec<$crate::token::Token> = $tokens.into();
      collected.extend(t_vec);
    )*
    $crate::tokens::Tokens::new(collected)
  }));
// We also need convenient auxiliaries, including auto-casting
impl From<Vec<Token>> for Tokens {
  fn from(ts: Vec<Token>) -> Tokens { Tokens::new(ts) }
}
impl From<Tokens> for Vec<Token> {
  fn from(ts: Tokens) -> Vec<Token> { ts.unlist() }
}

impl From<Token> for Tokens {
  fn from(t: Token) -> Tokens { Tokens::new(vec![t]) }
}
impl From<&Token> for Tokens {
  fn from(t: &Token) -> Tokens { Tokens::new(vec![t.clone()]) }
}

// Good news: Cloning `Token` should now be cheap (due to string interning),
// so cloning `Tokens` should be fine.
impl From<Rc<Tokens>> for Tokens {
  fn from(t: Rc<Tokens>) -> Tokens { (*t).clone() }
}
impl From<&Rc<Tokens>> for Tokens {
  fn from(t: &Rc<Tokens>) -> Tokens { (**t).clone() }
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
  fn from(mut ts: Tokens) -> Token {
    if ts.0.is_empty() {
      unimplemented!();
    } else if ts.0.len() == 1 {
      ts.0.remove(0)
    } else {
      panic!("Dangerous cast! Tokens->Token for {ts:?}");
      //let code = ts.0.first().unwrap().get_catcode();
      // Warn!("expected","token","multiple Tokens {:?} cast into a single Token: {:?}", ts,
      // single); Token::new(Cow::Owned(ts.to_string()), code)
    }
  }
}

impl<'a> From<&'a Tokens> for Token {
  fn from(ts: &'a Tokens) -> Token {
    if ts.0.is_empty() {
      unimplemented!();
    } else if ts.0.len() == 1 {
      ts.0.first().unwrap().clone()
    } else {
      panic!("Dangerous cast! Tokens->Token for {ts:?}");
      //let code = ts.0.first().unwrap().get_catcode();
      // Warn!("expected","token","multiple Tokens {:?} cast into a single Token: {:?}", ts,
      // single); Token::new(Cow::Owned(ts.to_string()), code)
    }
  }
}

impl From<Option<Tokens>> for Token {
  fn from(ts_opt: Option<Tokens>) -> Token {
    match ts_opt {
      Some(ts) => ts.into(),
      None => panic!("Casting a None (undef Tokens) into a Token is a Bug."),
    }
  }
}

impl From<Token> for Option<Tokens> {
  fn from(t: Token) -> Option<Tokens> { Some(Tokens::new(vec![t])) }
}
impl From<Token> for Option<Cow<'static, Tokens>> {
  fn from(t: Token) -> Option<Cow<'static, Tokens>> { Some(Cow::Owned(Tokens::new(vec![t]))) }
}

impl Display for Tokens {
  /// to_string is used often, and for more keyword-like reasons,
  /// NOT for creating valid TeX (use revert or UnTeX for that!)
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for t in &self.0 {
      write!(f, "{t}")?;
    }
    Ok(())
  }
}

impl AsRef<Tokens> for Tokens {
  fn as_ref(&self) -> &Tokens { self }
}

impl Tokens {
  /// Create a Tokens object from a `Vec` of individual `Token`
  pub fn new(tokens: Vec<Token>) -> Self { Tokens(tokens) }

  /// Return a list of the tokens making up this Tokens
  pub fn unlist(self) -> Vec<Token> { self.0 }

  /// Return a reference to the tokens making up this Tokens
  pub fn unlist_ref(&self) -> &Vec<Token> { &self.0 }

  /// Return a mutable reference to the tokens making up this Tokens
  pub fn unlist_mut(&mut self) -> &mut Vec<Token> { &mut self.0 }

  /// Are there any tokens at all contained in this Tokens object
  pub fn is_empty(&self) -> bool { self.0.is_empty() }

  /// Number of contained Token entries
  pub fn len(&self) -> usize { self.0.len() }

  /// Return a string containing the TeX form of the Tokens
  pub fn revert(self) -> Vec<Token> {
    self
      .0
      .into_iter()
      .map(|mut t| {
        if t.get_catcode() == Catcode::SmuggleTHE {
          *t.take_dont_expand().unwrap()
        } else {
          t
        }
      })
      .collect()
  }

  /// to_number casts back to a parsed Number (usually via gullet.read_number)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_number(&self) -> Number {
    if self.is_empty() {
      eprintln!("TODO: An empty tokens was requested for .to_number, debug this!");
      Number::default()
    } else {
      Number::new(self.to_string().parse::<i64>().unwrap_or(0))
    }
  }

  /// to_dimension casts back to a parsed Dimension (usually via gullet.read_dimension)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_dimension(&self) -> Dimension {
    // TODO: How do we enhance here to be able to use the current font information from State?
    // Using the State-ful variations makes it impossible to work with the From/Into standard Rust
    // traits. Should we do StatefulFrom/StatefulInto ?
    Dimension::new_f64(Dimension::spec_to_f64(&self.to_string(), None).unwrap_or_default())
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

  /// to_float casts back to a parsed Float (usually via gullet.read_float)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_float(&self) -> Float {
    if self.is_empty() {
      eprintln!("TODO: An empty tokens was requested for .to_float, debug this!");
      Float::default()
    } else {
      Float::new_f64(self.to_string().parse::<f64>().unwrap_or(0.0))
    }
  }

  /// to_keyvals casts back to a parsed KeyVals (usually via a KeyVals parameter type)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_keyvals(&self, state: &State) -> KeyVals {
    let mut toks_iter = self.unlist_ref().iter();
    let mut kvs = KeyVals::default();
    while let Some(key) = toks_iter.next() {
      key.with_str(|key_str| {
        if let Some(value) = toks_iter.next() {
          kvs.add_value(key_str, Stored::Token(value.clone()), false, false, state);
        } else {
          kvs.add_value(key_str, Stored::Tokens(Tokens!()), false, false, state);
        }
      });
    }
    kvs
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

  /// returns self, for compatibility convenience with `Option`
  pub fn unwrap_or_default(self) -> Tokens { self }
  /// returns self, for compatibility convenience with `Option`
  pub fn unwrap(&self) -> &Tokens { self }

  /// A string form which is primarily used for error-reporting
  pub fn stringify(&self) -> String {
    s!(
      "Tokens[{}]",
      &self
        .0
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
    )
  }
  /// digest the current `Tokens`
  pub fn be_digested(self, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
    stomach.digest(self, state)
  }

  /// Remove dont_expand, but preserve SMUGGLE_THE
  pub fn neutralize(self, extraspecials: &[char], state: &State) -> Tokens {
    Tokens(
      self
        .0
        .into_iter()
        .map(|t| t.neutralize(extraspecials, state))
        .collect::<Vec<_>>(),
    )
  }
  /// Checks if any BEGIN/END code groups are correctly nested and closed
  pub fn is_balanced(&self) -> bool {
    let mut level = 0;
    for t in &self.0 {
      level += match t.get_catcode() {
        Catcode::BEGIN => 1,
        Catcode::END => -1,
        _ => 0,
      };
      if level < 0 {
        // a negative level encountered at any point is ill-formed,
        // return early
        return false;
      }
    }
    level == 0
  }

  // NOTE: Assumes each arg either undef or also Tokens
  // Using inline accessors on those assumptions
  /// substitutes the parameters (ARG catcode) in a Tokens list for concrete arguments
  pub fn substitute_parameters(&self, args: &[Option<Cow<Tokens>>]) -> Self {
    let mut result = Vec::new();
    let in_tokens = self.0.iter();
    for token in in_tokens {
      if token.get_catcode() != Catcode::ARG {
        // Non-match; copy it
        result.push(token.clone());
      } else if let Some(ref arg) = args[&token.with_str(|ts| ts.parse::<usize>().unwrap()) - 1] {
        result.extend(arg.clone().into_owned().unlist());
      }
    }
    Tokens::new(result)
  }

  /// removes the smuggled token of all contained Token elements
  pub fn without_dont_expand(self) -> Self {
    Tokens(
      self
        .0
        .into_iter()
        .map(|t| t.without_dont_expand())
        .collect(),
    )
  }

  /// Consumes a Tokens to a string containing TeX that created it (or could have).
  /// Note that this is not necessarily the original TeX code; expansions or other substitutions may
  /// have taken place. Also note that the LaTeXML linebreak feature is always *disabled* here.)
  pub fn untex(self) -> String {
    let mut tokens: VecDeque<Token> = self.revert().into_iter().collect();
    let mut tex_string = String::new();
    let mut length = 0;
    let mut level = 0;
    let mut prevs = String::new();
    let mut prevcc = Catcode::COMMENT;
    while let Some(token) = tokens.pop_front() {
      let cc = token.get_catcode();
      if cc == Catcode::COMMENT {
        continue;
      }
      let mut token_string = token.to_string();
      // Note: \n only-used to fail alphanumeric test
      let first_char = token_string.chars().next().unwrap_or('\n');
      if cc == Catcode::LETTER {
        // keep "words" together, just for aesthetics
        while !tokens.is_empty() && tokens[0].get_catcode() == Catcode::LETTER {
          tokens
            .pop_front()
            .unwrap()
            .with_str(|front_str| token_string.push_str(front_str));
        }
      }

      let l = token_string.len();
      if cc == Catcode::BEGIN {
        level += 1;
      }
      //  Seems a reasonable & safe time to line break, for readability, etc.
      if cc == Catcode::SPACE && token_string == "\n" {
        // preserve newlines already present
        if length > 0 {
          tex_string.push_str(&token_string);
          length = 0;
        }
      // If this token is a letter (or otherwise starts with a letter or digit): space or linebreak
      } else {
        let last_prevs = prevs.chars().last().unwrap_or('_');
        // TOOD: this used to call "lookup_catcode" in State; is this char-check as good?
        let prev_is_letter = last_prevs.is_alphabetic();

        if (cc == Catcode::LETTER || (cc == Catcode::OTHER && first_char.is_alphanumeric()))
          && prevcc == Catcode::CS
          && prev_is_letter
        {
          // Insert a (virtual) space before a letter if previous token was a CS w/letters
          // This is required for letters, but just aesthetic for digits (to me?)
          let space = ' ';
          tex_string.push(space);
          tex_string.push_str(&token_string);
          length += 1 + l;
        } else {
          tex_string.push_str(&token_string);
          length += l;
        }
        if cc == Catcode::END {
          level -= 1;
        }
        prevs = token_string;
        prevcc = cc;
      }
    }
    // Patch up nesting for valid TeX !!!
    match level {
      1..=std::i32::MAX => {
        for _ in 0..level {
          tex_string.push('}');
        }
      },
      std::i32::MIN..=-1 => {
        for _ in 0..(-level) {
          tex_string = String::from("{") + &tex_string;
        }
      },
      0 => {},
    }
    tex_string
  }

  /// Process the `Catcode::PARAM` tokens for use as a macro body (and other token lists)
  // Groups PARAM+OTHER token pair into match tokens.
  // Collapses PARAM+PARAM token pair into a single PARAM
  // B book suggests running this
  // and remove dont_expand markers.
  pub fn pack_parameters(self) -> Self {
    let mut rescanned = Vec::new();
    let mut toks = self.unlist().into_iter().collect::<VecDeque<_>>();
    while let Some(mut t) = toks.pop_front() {
      if t.get_catcode() == Catcode::PARAM && !toks.is_empty() {
        // NOTE for future cleanup: Only CC_CS & CC_ACTIVE should ever get with_dont_expand!
        let next_t = toks.pop_front();
        let next_cc = next_t.as_ref().map(|t| t.get_catcode());
        if next_cc == Some(Catcode::OTHER) {
          // only group clear match token cases
          rescanned.push(Token {
            text: next_t.unwrap().get_sym(),
            code: Catcode::ARG,
            smuggled: None,
          });
        } else if next_cc == Some(Catcode::PARAM) {
          rescanned.push(t);
        } else {
          // any other case, preserve as-is, let the higher level call resolve any errors
          // e.g. \detokenize{#,} is legal, while \textbf{#,} is not
          Error!(
            "misdefined",
            "expansion",
            None,
            None,
            "Parameter has a malformed arg, should be #1-#9 or ##. In expansion {}",
            Tokens::new(toks.clone().into_iter().collect()).to_string()
          );
        }
      } else if let Some(mut inner) = t.take_dont_expand() {
        if let Some(smuggled) = inner.take_dont_expand() {
          rescanned.push(*smuggled);
        } else {
          rescanned.push(*inner);
        }
      } else {
        rescanned.push(t);
      }
    }
    Tokens::new(rescanned)
  }
}

impl ToTokens for Tokens {
  fn to_tokens(&self, stream: &mut TokenStream) {
    let d = &self.0;
    stream.extend(quote! {
        Tokens::new(<[Token]>::into_vec(Box::new([ #(#d),* ])))
    });
  }
}

impl ToTokens for Catcode {
  fn to_tokens(&self, stream: &mut TokenStream) {
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
      // Non-primitive
      IGNORE => "IGNORE",
      LETTER => "LETTER",
      OTHER => "OTHER",
      ACTIVE => "ACTIVE",
      COMMENT => "COMMENT",
      INVALID => "INVALID",
      CS => "CS",
      MARKER => "MARKER",
      ARG => "ARG",
      SmuggleTHE => "SmuggleTHE",
    };
    stream.append(Ident::new("Catcode", Span::call_site()));
    stream.append(Punct::new(':', Spacing::Joint));
    stream.append(Punct::new(':', Spacing::Alone));
    stream.append(Ident::new(kind, Span::call_site()));
  }
}

impl ToTokens for Token {
  fn to_tokens(&self, stream: &mut TokenStream) {
    let code = self.get_catcode();
    self.with_str(|text| {
      stream.extend(quote! {
        Token {
          text: rtx_core::common::arena::pin(#text),
          code: #code,
          smuggled: None
        }
      })
    });
  }
}
