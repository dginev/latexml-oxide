//! Token List constructors.
use crate::definition::argument::ArgWrap;
use crate::fmt;
use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt, quote};

use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt::Display;
use std::rc::Rc;

use crate::Digested;
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::float::Float;
use crate::common::glue::Glue;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::keyvals::KeyVals;
use crate::stomach;
use crate::token::*;

/// If untex is requested to add line-breaks, this is the line length it will allow
pub const UNTEX_LINELENGTH: usize = 78;
/// Use this to avoid reallocating a new empty Vec each time you need a placeholder Tokens return
/// value
pub const NO_TOKENS: Tokens = Tokens(Vec::new());
pub const NO_BORROWED_TOKENS: &Tokens = &NO_TOKENS;
/// Tokens are a thin wrapper over a vector of Token objects
///
/// They are usually read from a `Mouth` and treated as an immutable interface.
/// For access to the inner Token contents, use one of the `unlist` methods.
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
  fn from(t: &Token) -> Tokens { Tokens::new(vec![*t]) }
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
      // Match the &Tokens impl below: empty → \relax fallback rather
      // than panic. Callers that must see the empty case are rare and
      // should inspect Tokens directly.
      T_CS!("\\relax")
    } else if ts.0.len() == 1 {
      ts.0.remove(0)
    } else {
      // Prefer the first token and warn; cascading a panic here usually
      // means a stringly-typed binding slot received a multi-token value
      // (e.g. a macro argument coerced into a single-token slot). The
      // first token preserves TEx's "grab a single token" semantics.
      log::warn!("multi-token Tokens cast into single Token: {ts:?}");
      ts.0.remove(0)
    }
  }
}

impl<'a> From<&'a Tokens> for Token {
  fn from(ts: &'a Tokens) -> Token {
    if ts.0.is_empty() {
      T_CS!("\\relax") // empty Tokens → relax fallback
    } else if ts.0.len() == 1 {
      ts.0[0]
    } else {
      log::warn!("multi-token Tokens cast into single Token: {ts:?}");
      ts.0[0]
    }
  }
}

impl From<Option<Tokens>> for Token {
  fn from(ts_opt: Option<Tokens>) -> Token {
    match ts_opt {
      Some(ts) => ts.into(),
      None => T_CS!("\\relax"), // None → relax, matching the empty-Tokens path
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
      if t.code != Catcode::COMMENT {
        write!(f, "{t}")?;
      }
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

  /// Zero-alloc equivalent of `self.to_string().starts_with(prefix)`.
  /// Walks tokens byte-by-byte into `prefix` using the same Display
  /// semantics as `eq_text` (COMMENT skipped, ARG prefixed with `#`).
  /// Returns `true` once the full prefix has been consumed, even if
  /// more token text follows.
  pub fn starts_with_text(&self, prefix: &str) -> bool {
    let mut remaining = prefix;
    for t in &self.0 {
      if remaining.is_empty() {
        return true;
      }
      if t.code == crate::token::Catcode::COMMENT {
        continue;
      }
      if t.code == crate::token::Catcode::ARG {
        if !remaining.starts_with('#') {
          return false;
        }
        remaining = &remaining[1..];
        if remaining.is_empty() {
          return true;
        }
      }
      let keep_going = t.with_str(|text| {
        if text.is_empty() {
          return true;
        }
        if remaining.starts_with(text) {
          remaining = &remaining[text.len()..];
          true
        } else if text.starts_with(remaining) {
          // This token's text extends past `prefix` — prefix matches
          // and we're done.
          remaining = "";
          true
        } else {
          false
        }
      });
      if !keep_going {
        return false;
      }
      if remaining.is_empty() {
        return true;
      }
    }
    remaining.is_empty()
  }

  /// Zero-alloc equivalent of `self.to_string() == target`. Walks the
  /// contained tokens byte-by-byte, skipping COMMENT tokens (matching
  /// `Display for Tokens`) and prefixing ARG tokens with `#` (matching
  /// `Display for Token`). Returns `true` iff the rendered text exactly
  /// equals `target`. Used by DefMacro bodies that check keyword
  /// values like `true` / `false` / `swapnumber` without wanting to
  /// allocate a fresh `String` per invocation.
  pub fn eq_text(&self, target: &str) -> bool {
    let mut remaining = target;
    for t in &self.0 {
      if t.code == crate::token::Catcode::COMMENT {
        continue;
      }
      if t.code == crate::token::Catcode::ARG {
        if !remaining.starts_with('#') {
          return false;
        }
        remaining = &remaining[1..];
      }
      let ok = t.with_str(|text| {
        if remaining.starts_with(text) {
          remaining = &remaining[text.len()..];
          true
        } else {
          false
        }
      });
      if !ok {
        return false;
      }
    }
    remaining.is_empty()
  }

  // Just a synonym for unlist in this reversion case
  pub fn revert(self) -> Vec<Token> { self.0 }

  /// to_number casts back to a parsed Number (usually via gullet::read_number)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_number(&self) -> Number {
    if self.is_empty() {
      log::debug!("to_number called on empty Tokens — returning 0 (TeX-compatible default)");
      Number::default()
    } else {
      Number::new(self.to_string().parse::<i64>().unwrap_or(0))
    }
  }

  /// to_dimension casts back to a parsed Dimension (usually via gullet::read_dimension)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_dimension(&self) -> Dimension {
    // TODO: How do we enhance here to be able to use the current font information from state::
    // Using the state::ful variations makes it impossible to work with the From/Into standard Rust
    // traits. Should we do stateful From/Into ?
    Dimension::new_f64(Dimension::spec_to_f64(&self.to_string()).unwrap_or_default())
  }

  /// to_glue casts back to a parsed Glue (usually via gullet::read_glue)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_glue(&self) -> Glue {
    let token: Token = self.into();
    token.to_glue()
  }

  /// to_mu_glue casts back to a parsed MuGlue (usually via gullet::read_mu_glue)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_mu_glue(&self) -> MuGlue {
    let token: Token = self.into();
    token.to_mu_glue()
  }

  /// to_mu_dimension casts back to a parsed MuGlue (usually via gullet::read_mu_glue)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_mu_dimension(&self) -> MuDimension {
    let token: Token = self.into();
    token.to_mu_dimension()
  }

  /// to_float casts back to a parsed Float (usually via gullet::read_float)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_float(&self) -> Float {
    if self.is_empty() {
      log::debug!("to_float called on empty Tokens — returning 0.0 (TeX-compatible default)");
      Float::default()
    } else {
      Float::new_f64(self.to_string().parse::<f64>().unwrap_or(0.0))
    }
  }

  /// to_keyvals casts back to a parsed KeyVals (usually via a KeyVals parameter type)
  /// which had to be re-converted to a Tokens for reentering the expansion flow
  pub fn to_keyvals(&self) -> Result<KeyVals> {
    let mut toks_iter = self.unlist_ref().iter();
    let mut kvs = KeyVals::default();
    while let Some(key) = toks_iter.next() {
      key.with_str(|key_str| {
        if let Some(value) = toks_iter.next() {
          kvs.add_value(key_str, ArgWrap::Token(*value), false, false)
        } else {
          kvs.add_value(key_str, ArgWrap::Tokens(Tokens!()), false, false)
        }
      })?;
    }
    Ok(kvs)
  }

  /// Methods for overloaded ops.
  pub fn equals(&self, other: Tokens) -> bool {
    let self_tokens: Vec<&Token> = self
      .0
      .iter()
      .filter(|t| t.code != Catcode::COMMENT && t.code != Catcode::MARKER)
      .collect();
    let other_tokens: Vec<&Token> = other
      .0
      .iter()
      .filter(|t| t.code != Catcode::COMMENT && t.code != Catcode::MARKER)
      .collect();
    if self_tokens.len() != other_tokens.len() {
      false
    } else {
      self_tokens
        .into_iter()
        .zip(other_tokens)
        .all(|(t_self, t_other)| *t_self == *t_other)
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
  pub fn be_digested(self) -> Result<Digested> { stomach::digest(self) }

  /// neutralize each token
  pub fn neutralize(self, extraspecials: &[char]) -> Tokens {
    Tokens(
      self
        .0
        .into_iter()
        .map(|t| t.neutralize(extraspecials))
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
    // Pre-size: the substituted result is at least as long as the
    // template. Expansion bodies can be thousands of tokens in the
    // expl3 kernel; pre-allocation skips the first several Vec doublings.
    let mut result = Vec::with_capacity(self.0.len());
    for token in self.0.iter() {
      if token.get_catcode() != Catcode::ARG {
        // Non-match; copy it
        result.push(*token);
      } else {
        let idx = token.with_str(|ts| ts.parse::<usize>().unwrap_or(0));
        if idx > 0 && idx <= args.len() {
          if let Some(ref arg) = args[idx - 1] {
            // `arg` is `Cow<Tokens>`; iterate via `unlist_ref` + copy
            // (Tokens is a Vec<Token> of `Copy` tokens). Avoids the
            // previous `clone().into_owned().unlist()` chain which
            // double-cloned the Vec when `arg` was `Cow::Borrowed`.
            result.extend(arg.as_ref().unlist_ref().iter().copied());
          }
        }
      }
    }
    Tokens::new(result)
  }

  /// Consumes a Tokens to a string containing TeX that created it (or could have).
  /// Note that this is not necessarily the original TeX code; expansions or other substitutions may
  /// have taken place.
  ///
  /// **Design decision:** The Perl `UnTeX` inserts `%\n` line-breaks (TeX comment + newline) when
  /// a token string would exceed 78 characters. The Rust port deliberately omits this feature.
  /// Line-break insertion is purely cosmetic and makes test expectations fragile — the `%\n`
  /// appears verbatim in `tex=` attributes of `ltx:Math` elements, causing test XML files to
  /// contain `%&#10;` escape sequences that depend on exact token lengths. We instead always
  /// produce compact, single-line output. Test `.xml` files should not contain `%&#10;`.
  pub fn untex(self) -> String {
    // `VecDeque::from(Vec)` reuses the Vec's heap buffer directly
    // (no second allocation), unlike `.into_iter().collect()`.
    let mut tokens: VecDeque<Token> = VecDeque::from(self.revert());
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
        // Perl: $STATE->lookupCatcode($1) == CC_LETTER
        // Must use actual catcode lookup, not just is_alphabetic(), because
        // characters like @ may have catcode LETTER in some contexts.
        let prev_is_letter = crate::state::lookup_catcode(last_prevs)
          .map(|cc| cc == Catcode::LETTER)
          .unwrap_or_else(|| last_prevs.is_alphabetic());

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
      1..=i32::MAX => {
        for _ in 0..level {
          tex_string.push('}');
        }
      },
      i32::MIN..=-1 => {
        // Prepend `-level` opening braces in one alloc (was O(n²) with
        // String::from("{") + &tex_string per iteration).
        let n = (-level) as usize;
        let mut prefixed = String::with_capacity(n + tex_string.len());
        for _ in 0..n {
          prefixed.push('{');
        }
        prefixed.push_str(&tex_string);
        tex_string = prefixed;
      },
      0 => {},
    }
    tex_string
  }

  /// Packs repeated CC_PARAM tokens into CC_ARG tokens for use as a macro body (and other token
  /// lists) Also unwraps \noexpand tokens, since that is also needed for macro bodies
  /// (but not strictly part of packing parameters)
  pub fn pack_parameters(self) -> Result<Self> {
    // Result is at most the same size as input (param-digit pairs
    // collapse 2→1; other tokens copy 1→1). Pre-sizing avoids the
    // initial Vec doublings on 1k+ token expansions (common for
    // expl3 macros).
    let mut rescanned = Vec::with_capacity(self.0.len());
    // `VecDeque::from(Vec)` reuses the Vec's heap buffer directly (no
    // second allocation), unlike `into_iter().collect()` which copies.
    let mut toks: VecDeque<Token> = VecDeque::from(self.unlist());
    while let Some(t) = toks.pop_front() {
      if t.get_catcode() == Catcode::PARAM && !toks.is_empty() {
        let next_t = toks.pop_front();
        let next_cc = next_t.as_ref().map(|t| t.get_catcode());
        if next_cc == Some(Catcode::OTHER) {
          // only group clear match token cases
          rescanned.push(Token {
            text: next_t.unwrap().get_sym(),
            code: Catcode::ARG,
          });
        } else if next_cc == Some(Catcode::PARAM) {
          rescanned.push(t);
        } else {
          // any other case, preserve as-is, let the higher level call resolve any errors
          // e.g. \detokenize{#,} is legal, while \textbf{#,} is not
          // Note: this also fires for alignment templates (\halign{#\hfil&...}) which is valid TeX.
          // Perl has the same warning (Tokens.pm packParameters line 139). Non-fatal.
          Error!(
            "misdefined",
            "expansion",
            "Parameter has a malformed arg, should be #1-#9 or ##. In expansion {}",
            Tokens::new(toks.clone().into_iter().collect()).to_string()
          );
        }
      } else {
        rescanned.push(t);
      }
    }
    Ok(Tokens::new(rescanned))
  }

  /// Trims outer braces (if they balance each other).
  /// Strips exactly 1 layer of matching outer braces by default.
  /// Should this also trim whitespace? or only if there are braces?
  pub fn strip_braces(self) -> Self { self.strip_braces_n(1) }

  /// Trims `layers` outer brace pairs (if they balance each other).
  /// Also trims whitespace *outer to* the removed braces.
  /// Follows the Perl Tokens.pm algorithm: first collects all balanced
  /// brace pairs, then strips from outside-in, only removing pairs that
  /// span the full remaining width.
  pub fn strip_braces_n(self, mut layers: usize) -> Self {
    let tokens = self.0;
    let n = tokens.len();
    if n <= 1 {
      return Tokens::new(tokens);
    }

    let mut i0: usize = 0;
    let mut i1: usize = n;

    // skip past spaces at ends
    while i0 < i1 && tokens[i0].get_catcode() == Catcode::SPACE {
      i0 += 1;
    }
    while i1 > i0 && tokens[i1 - 1].get_catcode() == Catcode::SPACE {
      i1 -= 1;
    }

    // Collect balanced pairs (innermost first due to stack order)
    let mut opens: Vec<usize> = Vec::new();
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in i0..i1 {
      match tokens[i].get_catcode() {
        Catcode::BEGIN => opens.push(i),
        Catcode::END => {
          if let Some(j) = opens.pop() {
            pairs.push((j, i));
          } else {
            return Tokens::new(tokens); // Unbalanced: Too many }
          }
        },
        _ => {},
      }
    }
    if !opens.is_empty() {
      return Tokens::new(tokens); // Unbalanced: Too many {
    }

    // Strip layers from outside-in.
    // pairs is ordered innermost-first, so pop() gives outermost pair first.
    while layers > 0 {
      layers -= 1;
      if let Some((j0, j1)) = pairs.pop() {
        if j0 == i0 && j1 == i1 - 1 {
          i0 += 1;
          i1 -= 1;
        }
      }
    }

    // Empty after stripping
    if i0 >= i1 {
      return Tokens::new(Vec::new());
    }

    if i0 > 0 || i1 < n {
      Tokens::new(tokens[i0..i1].to_vec())
    } else {
      Tokens::new(tokens)
    }
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
          text: latexml_core::common::arena::pin_static(#text),
          code: #code
        }
      })
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::arena;

  fn letter_tok(s: &str) -> Token {
    Token {
      text: arena::pin(s),
      code: Catcode::LETTER,
    }
  }

  fn comment_tok(s: &str) -> Token {
    Token {
      text: arena::pin(s),
      code: Catcode::COMMENT,
    }
  }

  #[test]
  fn empty_tokens_len_zero() {
    let t = Tokens::new(vec![]);
    assert_eq!(t.len(), 0);
    assert!(t.is_empty());
  }

  #[test]
  fn tokens_new_preserves_order() {
    let t = Tokens::new(vec![letter_tok("a"), letter_tok("b"), letter_tok("c")]);
    assert_eq!(t.len(), 3);
    let list = t.unlist();
    let texts: Vec<String> = list.iter().map(|t| arena::to_string(t.text)).collect();
    assert_eq!(texts, vec!["a", "b", "c"]);
  }

  #[test]
  fn tokens_unlist_ref_does_not_consume() {
    let t = Tokens::new(vec![letter_tok("a")]);
    let r = t.unlist_ref();
    assert_eq!(r.len(), 1);
    // t is still usable after unlist_ref.
    assert_eq!(t.len(), 1);
  }

  #[test]
  fn tokens_stringify_format() {
    let t = Tokens::new(vec![letter_tok("a"), letter_tok("b")]);
    let s = t.stringify();
    assert!(s.starts_with("Tokens["), "got {s:?}");
    assert!(s.ends_with(']'));
    assert!(s.contains("a"));
    assert!(s.contains("b"));
  }

  #[test]
  fn tokens_equals_ignores_comments_and_markers() {
    // equals() filters out COMMENT and MARKER tokens before comparing.
    let a = Tokens::new(vec![letter_tok("x"), comment_tok("%"), letter_tok("y")]);
    let b = Tokens::new(vec![letter_tok("x"), letter_tok("y")]);
    assert!(a.equals(b), "comments should be ignored in equals()");
  }

  #[test]
  fn tokens_equals_different_content() {
    let a = Tokens::new(vec![letter_tok("x")]);
    let b = Tokens::new(vec![letter_tok("y")]);
    assert!(!a.equals(b));
  }

  #[test]
  fn tokens_equals_different_lengths() {
    let a = Tokens::new(vec![letter_tok("x")]);
    let b = Tokens::new(vec![letter_tok("x"), letter_tok("y")]);
    assert!(!a.equals(b));
  }

  #[test]
  fn tokens_equals_both_empty() {
    let a = Tokens::new(vec![]);
    let b = Tokens::new(vec![]);
    assert!(a.equals(b));
  }

  #[test]
  fn tokens_unwrap_self_identity() {
    let t = Tokens::new(vec![letter_tok("x")]);
    assert_eq!(t.unwrap().len(), 1);
  }

  #[test]
  fn tokens_revert_returns_vec() {
    let t = Tokens::new(vec![letter_tok("x"), letter_tok("y")]);
    let v = t.revert();
    assert_eq!(v.len(), 2);
  }

  #[test]
  fn tokens_display_joins_content() {
    // Display on Tokens concatenates each token's Display.
    let t = Tokens::new(vec![letter_tok("a"), letter_tok("b"), letter_tok("c")]);
    let s = format!("{t}");
    assert_eq!(s, "abc");
  }
}
