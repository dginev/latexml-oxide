//! Token List constructors.
use std::{borrow::Cow, collections::VecDeque, fmt::Display, rc::Rc};

#[cfg(feature = "codegen")]
use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream};
#[cfg(feature = "codegen")]
use quote::{ToTokens, TokenStreamExt, quote};

use crate::{
  Digested,
  common::{
    dimension::Dimension,
    error::{emit_warn, *},
    float::Float,
    glue::Glue,
    mudimension::MuDimension,
    muglue::MuGlue,
    number::Number,
    numeric_ops::NumericOps,
  },
  definition::argument::ArgWrap,
  fmt,
  keyvals::KeyVals,
  stomach,
  token::*,
};

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
  ($( $tokens:expr_2021 ),+) => ({
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
      emit_warn(
        "internal",
        "tokens",
        &format!("multi-token Tokens cast into single Token: {ts:?}"),
      );
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
      emit_warn(
        "internal",
        "tokens",
        &format!("multi-token Tokens cast into single Token: {ts:?}"),
      );
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

/// A string of **TeX markup** — text that is safe to hand back to the tokenizer.
///
/// It is *not* a path and *not* an input `.tex` file (`source_directory`,
/// `--source-map` and `docs/performance/SOURCE_PROVENANCE.md` own that sense of
/// "source"); it is the character content a [`crate::mouth::Mouth`] will read.
///
/// # Why the type exists
///
/// Flattening [`struct@Tokens`] with [`Display`] **welds control words**. TeX consumes
/// the space that terminates a control word, so `\v S` tokenizes to `[\v][S]`;
/// re-emitting that with `Display` gives `\vS`, a control sequence that exists in
/// no LaTeX. [`Tokens::untex`] re-emits the space, `Display` deliberately does
/// not — this is faithful to Perl (`Core/Tokens.pm:61 toString` joins the token
/// strings, and `Core/Token.pm:306` returns a CS name with no trailing space),
/// whose own comment says the result is "NOT for creating valid TeX (use revert
/// or UnTeX for that!)".
///
/// Perl relies on author discipline there. It has failed three times in this
/// port — `\bib@@names` (PR #399), `dcolumn`/`overpic` (PR #400), and the
/// MathSciNet review path (issue 410: `MRREVIEWER = {Fran\c cois\ Digne}` became
/// `undefined:\ccois`) — each found by a user-visible failure years after the
/// code was written. `TeXString` makes the mistake unrepresentable instead: the
/// tokenizing sinks take `impl Into<TeXString>`, and a bare `String` has no way
/// in.
///
/// # The three ways in
///
/// * `From<&'static str>` — a string *literal* in a binding is TeX its author
///   typed by hand, so it converts implicitly and the ~125 literal call sites
///   stay untouched. There is deliberately **no** `From<String>` and **no**
///   `From<&str>`: those are exactly the shapes a welded `Tokens::to_string()`
///   arrives in, and `s!(…)`/`format!(…)` returns the former.
/// * [`Tokens::untex_string`] — the blessed path from `Tokens`.
/// * [`TeXString::assembled`] — the explicit escape hatch, for a `format!` of
///   literal TeX around already-safe pieces. It names the obligation it imposes.
///
/// ```
/// # use latexml_core::tokens::TeXString;
/// let s: TeXString = r"\relax".into(); // literal: implicit
/// assert_eq!(s.as_str(), r"\relax");
/// ```
///
/// A `String` cannot get in on its own — this is the guard, and it bites:
///
/// ```compile_fail
/// # use latexml_core::tokens::TeXString;
/// let welded: String = String::from(r"\vS");
/// let _: TeXString = welded.into(); // no `From<String>`: does not compile
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct TeXString(Cow<'static, str>);

impl TeXString {
  /// Assert that an owned `String` is valid TeX markup.
  ///
  /// **The caller's obligation**: every interpolated fragment must be either
  /// literal TeX written at the call site, or a fragment that came from
  /// [`Tokens::untex_string`] / another `TeXString`. It must **not** be a bare
  /// `Tokens::to_string()` — that is the welding bug this type exists to
  /// prevent (`\v S` → `\vS`); use [`Tokens::untex_string`] for those.
  ///
  /// The typical honest use is a `format!` whose *shape* is literal TeX:
  ///
  /// ```
  /// # use latexml_core::tokens::TeXString;
  /// let counter = "section";
  /// let tex = TeXString::assembled(format!(r"\the{counter}"));
  /// assert_eq!(tex.as_str(), r"\thesection");
  /// ```
  pub fn assembled(tex: String) -> Self { TeXString(Cow::Owned(tex)) }

  /// The TeX markup, borrowed.
  pub fn as_str(&self) -> &str { &self.0 }

  /// The TeX markup, owned (allocates only when this was built from a literal).
  pub fn into_string(self) -> String { self.0.into_owned() }

  /// Is there any markup at all?
  pub fn is_empty(&self) -> bool { self.0.is_empty() }
}

impl From<&'static str> for TeXString {
  /// A `&'static str` in a binding is a TeX literal its author typed by hand.
  ///
  /// Deliberately the *only* blanket string conversion — see the type docs.
  fn from(tex: &'static str) -> Self { TeXString(Cow::Borrowed(tex)) }
}

impl Display for TeXString {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str(&self.0) }
}

impl AsRef<str> for TeXString {
  fn as_ref(&self) -> &str { &self.0 }
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
      if t.code == Catcode::COMMENT {
        continue;
      }
      if t.code == Catcode::ARG {
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
      if t.code == Catcode::COMMENT {
        continue;
      }
      if t.code == Catcode::ARG {
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
        if idx > 0
          && idx <= args.len()
          && let Some(ref arg) = args[idx - 1]
        {
          // `arg` is `Cow<Tokens>`; iterate via `unlist_ref` + copy
          // (Tokens is a Vec<Token> of `Copy` tokens). Avoids the
          // previous `clone().into_owned().unlist()` chain which
          // double-cloned the Vec when `arg` was `Cow::Borrowed`.
          result.extend(arg.as_ref().unlist_ref().iter().copied());
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

  /// [`untex`](Tokens::untex), typed — the blessed way to get TeX markup out of
  /// a `Tokens` and back into a tokenizing sink.
  ///
  /// Prefer this over `untex()` whenever the string is destined for
  /// `Tokenize!` / `mouth::tokenize` / `mouth::tokenize_internal`: it is the
  /// only [`struct@Tokens`]→[`TeXString`] conversion, so the sink's signature proves
  /// the round trip cannot weld a control word (`\v S` stays `\v S`, not
  /// `\vS`). `untex()` itself is unchanged for the callers that want a plain
  /// `String` (a `tex=` attribute, a log message, a comparison).
  pub fn untex_string(self) -> TeXString { TeXString::assembled(self.untex()) }

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
    // tex.web resolves tokens by MEANING during macro-definition scanning
    // (get_next assigns cur_cmd=mac_param for a CS `\let` to a catcode-6
    // `#`), so an IMPLICIT parameter token participates in `#1`/`##`
    // pairing exactly like a literal `#`. Generated code relies on it:
    // ctexart.cls L1194 `\cs_new_protected:Npn \__ctex_patch_toc_width:n
    // \c_parameter_token 1 { … \c_parameter_token 1 … }` (docstrip writes
    // the CS spelling to survive catcode changes) — without this, the CS
    // leaks literally into hook names → `\csname __hook package/#…` errors
    // across the 50-doc ctex family.
    let is_param_tok = |t: &Token| {
      t.get_catcode() == Catcode::PARAM
        || (t.get_catcode().is_active_or_cs()
          && matches!(crate::state::lookup_meaning(t),
                      Some(crate::common::store::Stored::Token(l))
                        if l.get_catcode() == Catcode::PARAM))
    };
    while let Some(mut t) = toks.pop_front() {
      if t.get_catcode() != Catcode::PARAM && is_param_tok(&t) {
        t = T_PARAM!();
      }
      if t.get_catcode() == Catcode::PARAM && !toks.is_empty() {
        let next_t = toks.pop_front();
        let next_cc = next_t.as_ref().map(|t| t.get_catcode());
        if next_cc == Some(Catcode::OTHER) {
          // only group clear match token cases
          rescanned.push(Token {
            text: next_t.unwrap().get_sym(),
            code: Catcode::ARG,
            #[cfg(feature = "token-locators")]
            loc: 0,
          });
        } else if next_cc == Some(Catcode::PARAM)
          || next_t.as_ref().map(&is_param_tok) == Some(true)
        {
          rescanned.push(t);
        } else {
          // A PARAM (`#`) followed by neither a digit nor another `#` is, in
          // real documents, almost always a `\halign`/`\valign` alignment-cell
          // marker embedded in a macro body (e.g. `\def\foo{\halign{#\hfil&...}}`)
          // or the `#{` end-of-parameter-text delimiter — both VALID TeX where
          // the catcode-6 `#` must survive losslessly into the template/preamble.
          // Real TeX resolves the parameter-vs-cell ambiguity during alignment
          // processing, a lower level than LaTeXML operates at, so we cannot
          // reliably tell this apart from a genuine typo.
          //
          // Perl's packParameters (Tokens.pm L139) emits a *counted* Error here
          // AND drops both tokens, corrupting the template — but Perl rarely
          // reaches it (it often can't find the offending package and skips the
          // raw load). We DO raw-load such packages, so erroring+dropping broke
          // the error-free target for the common halign-in-macro idiom. Preserve
          // both tokens and log at Info (non-counted) instead. Documented as a
          // beneficial divergence in docs/parity/KNOWN_PERL_ERRORS.md item 1. Witness
          // 2006.02269 (easyeqn.sty `{MATRIX}` env → `$\mathstrut##$` template;
          // 2 errors → 0).
          Info!(
            "misdefined",
            "expansion",
            "Lone # (catcode PARAM) preserved as alignment/template marker. In expansion {}",
            Tokens::new(toks.clone().into_iter().collect()).to_string()
          );
          rescanned.push(t);
          if let Some(nt) = next_t {
            rescanned.push(nt);
          }
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
      if let Some((j0, j1)) = pairs.pop()
        && j0 == i0
        && j1 == i1 - 1
      {
        i0 += 1;
        i1 -= 1;
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

// `impl ToTokens` blocks below are gated on the `codegen` feature
// (audit DEP-14, 2026-05-18). They are called only at compile time by
// `latexml_codegen` proc-macros via `quote!{ ... #tokens_value ... }`
// splices. Resolver v2 keeps proc-macro feature unification isolated,
// so the runtime `latexml_core` linked into `latexml_oxide` does NOT
// compile these impls — dropping `proc-macro2` (~93 KiB) and `quote`
// from the runtime binary's dependency graph.
#[cfg(feature = "codegen")]
impl ToTokens for Tokens {
  fn to_tokens(&self, stream: &mut TokenStream) {
    let d = &self.0;
    stream.extend(quote! {
        Tokens::new(<[Token]>::into_vec(Box::new([ #(#d),* ])))
    });
  }
}

#[cfg(feature = "codegen")]
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

#[cfg(feature = "codegen")]
impl ToTokens for Token {
  fn to_tokens(&self, stream: &mut TokenStream) {
    let code = self.get_catcode();
    self.with_str(|text| {
      stream.extend(quote! {
        Token {
          text: latexml_core::common::arena::pin_static(#text),
          code: #code,
          // Emitted into the consumer crate; the cfg resolves there (the feature
          // propagates from latexml_oxide). See docs/performance/SOURCE_PROVENANCE.md §3.1.1.
          #[cfg(feature = "token-locators")]
          loc: 0u32
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
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  }

  fn comment_tok(s: &str) -> Token {
    Token {
      text: arena::pin(s),
      code: Catcode::COMMENT,
      #[cfg(feature = "token-locators")]
      loc: 0,
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

#[cfg(test)]
mod untex_control_word_space_tests {
  use super::*;

  /// `untex()` must re-emit the space that terminates a control word;
  /// `to_string()` documents that it does not, and the two must not be
  /// confused at a call site that needs valid TeX back.
  ///
  /// TeX CONSUMES the space after a control word, so by token-time it is gone
  /// as data — `\v S` and `\vS` tokenize to different things but a naive
  /// concatenation of the first yields the second. `\vS` exists in no LaTeX.
  ///
  /// This is not academic: `\bib@@names` used `to_string()` where Perl uses
  /// `UnTeX` (BibTeX.pool.ltxml L277), which mangled every space-form accent in
  /// a bibliography author name — `{\v S}`, `{\c c}`, `{\" a}`, i.e. most
  /// non-English names — into an undefined macro. ~+2800 error documents per
  /// corpus on the 2026-07-26 sandbox sweep.
  #[test]
  fn untex_reemits_the_space_that_terminates_a_control_word() {
    for (src, expect_untex) in [
      (r"\v Spakov", r"\v Spakov"), // control word + space + LETTER: space needed
      (r"\c calves", r"\c calves"), //   likewise
      (r"\v{S}pakov", r"\v{S}pakov"), // braced argument: no space needed, none added
    ] {
      let toks = crate::mouth::tokenize(src);
      assert_eq!(
        toks.clone().untex(),
        expect_untex,
        "untex({src:?}) must round-trip to valid TeX"
      );
      // Re-tokenizing the untex output must yield the SAME control sequence —
      // the property that actually matters, and the one that broke.
      let first = |t: Tokens| {
        t.unlist()
          .first()
          .map(|t| t.to_string())
          .unwrap_or_default()
      };
      assert_eq!(
        first(crate::mouth::tokenize(toks.clone().untex_string())),
        first(crate::mouth::tokenize(src)),
        "untex({src:?}) changed the leading control sequence on re-tokenization"
      );
    }
  }

  /// The documented contract of the other direction, pinned so nobody
  /// "helpfully" makes `Display` TeX-correct and silently changes every
  /// keyword-ish `to_string()` caller in the tree.
  #[test]
  fn to_string_deliberately_does_not_reemit_the_space() {
    let toks = crate::mouth::tokenize(r"\v Spakov");
    assert_eq!(
      toks.to_string(),
      r"\vSpakov",
      "Display for Tokens is documented as NOT producing valid TeX; if this \
       changes, audit every to_string() caller before updating the expectation"
    );
  }
}

/// The guard itself: proof, at COMPILE time, that a welded `String` cannot reach
/// a tokenizing sink.
///
/// This module contains no runtime assertions worth the name — its whole value is
/// that it stops compiling if the property is lost. It is `#[cfg(test)]`, so it is
/// checked whenever the test target is built (`cargo test --tests`, and so CI).
#[cfg(test)]
mod texstring_guard_tests {
  use super::*;

  /// Neither `String: Into<TeXString>` nor `&String: Into<TeXString>` may hold.
  ///
  /// Those are the shapes a control-word-welding `Tokens::to_string()` arrives in
  /// (`s!(…)`/`format!(…)` gives the first, `&some_local` the second), so an
  /// implicit conversion would silently reopen the bug this type exists to close
  /// (`\bib@@names` PR #399, `dcolumn`/`overpic` PR #400, MathSciNet review path
  /// issue 410).
  ///
  /// Mechanism (the `static_assertions::assert_not_impl_any!` trick, hand-rolled
  /// to avoid the dependency): a helper trait with a blanket impl for everything
  /// plus a second impl gated on `Into<TeXString>`. If BOTH apply the item path is
  /// ambiguous and this fails to compile; if only the blanket one applies it
  /// resolves. So "still compiles" == "the conversion does not exist".
  ///
  /// A `&str` whose lifetime is shorter than `'static` is rejected too, but not
  /// by this assertion — an unconstrained `&'_ str` here infers to `&'static str`,
  /// which is exactly the case that MUST convert. The compiler enforces that half
  /// at the call site instead, as an E0521 "borrowed data escapes … `'1` must
  /// outlive `'static`".
  const _NO_IMPLICIT_STRING_CONVERSION: fn() = || {
    trait AmbiguousIfConvertible<A> {
      fn some_item() {}
    }
    impl<T> AmbiguousIfConvertible<()> for T {}
    impl<T: Into<TeXString>> AmbiguousIfConvertible<u8> for T {}

    let _ = <String as AmbiguousIfConvertible<_>>::some_item;
    let _ = <&String as AmbiguousIfConvertible<_>>::some_item;
  };

  /// The other half: a `&'static str` — i.e. every TeX literal a binding types
  /// out — must keep converting implicitly, or ~125 call sites would need
  /// ceremony for nothing.
  const _LITERALS_STILL_CONVERT: fn() = || {
    fn sink(_: impl Into<TeXString>) {}
    sink(r"\relax");
    sink(TeXString::assembled(String::new()));
  };

  /// …while the three blessed ways in must all still work. Kept beside the
  /// negative assertion so a "fix" that deletes the conversions to satisfy it is
  /// caught here.
  #[test]
  fn the_three_blessed_constructors_reach_the_sink() {
    // 1. a literal
    assert_eq!(crate::mouth::tokenize(r"\relax").to_string(), r"\relax");
    // 2. untex_string — the space that terminates `\v` survives the round trip
    let welded = crate::mouth::tokenize(r"\v Spakov");
    assert_eq!(
      crate::mouth::tokenize(welded.clone().untex_string()).to_string(),
      r"\vSpakov",
      "re-tokenizing untex_string() output must give back the SAME tokens \
       (whose Display is again the welded form) — i.e. \\v stayed \\v"
    );
    // …which is precisely what the welded string does NOT do:
    let welded_again = crate::mouth::tokenize(TeXString::assembled(welded.to_string()));
    assert_eq!(
      welded_again.unlist().first().map(|t| t.to_string()),
      Some(r"\vSpakov".to_string()),
      "the welded path collapses \\v + Spakov into the single undefined CS \
       \\vSpakov — the bug TeXString exists to make hard to write"
    );
    // 3. assembled
    assert_eq!(
      crate::mouth::tokenize(TeXString::assembled(format!(r"\the{}", "section"))).to_string(),
      r"\thesection"
    );
  }
}
