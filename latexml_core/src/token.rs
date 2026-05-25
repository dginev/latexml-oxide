use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use crate::Digested;
use crate::common::arena::{self, SymStr};
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::float::Float;
use crate::common::glue::Glue;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::common::store::Stored;
use crate::definition::Definition;
use crate::definition::register::Register;
use crate::state;
use crate::tokens::Tokens;

static CONTROLNAME: &[&str] = &[
  "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL", "BS", "HT", "LF", "VT", "FF", "CR", "SO",
  "SI", "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN", "EM", "SUB", "ESC", "FS",
  "GS", "RS", "US",
];

/// A Token category code, as in TeX
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Catcode {
  ESCAPE,
  BEGIN,
  END,
  MATH,
  ALIGN,
  EOL,
  PARAM,
  SUPER,
  SUB,
  IGNORE,
  SPACE,
  LETTER,
  OTHER,
  ACTIVE,
  COMMENT,
  INVALID,
  CS,
  MARKER,
  ARG,
}

impl From<u8> for Catcode {
  fn from(num: u8) -> Catcode {
    use crate::token::Catcode::*;
    match num {
      0 => ESCAPE,
      1 => BEGIN,
      2 => END,
      3 => MATH,
      4 => ALIGN,
      5 => EOL,
      6 => PARAM,
      7 => SUPER,
      8 => SUB,
      9 => IGNORE,
      10 => SPACE,
      11 => LETTER,
      12 => OTHER,
      13 => ACTIVE,
      14 => COMMENT,
      15 => INVALID,
      16 => CS,
      17 => MARKER,
      18 => ARG,
      _ => {
        // let message = s!("Unrecognized catcode: {:?}", num);
        // Warn!("unknown", "catcode", None, message);
        IGNORE
      },
    }
  }
}

impl From<Catcode> for u8 {
  fn from(cc: Catcode) -> u8 {
    use crate::token::Catcode::*;
    match cc {
      ESCAPE => 0,
      BEGIN => 1,
      END => 2,
      MATH => 3,
      ALIGN => 4,
      EOL => 5,
      PARAM => 6,
      SUPER => 7,
      SUB => 8,
      IGNORE => 9,
      SPACE => 10,
      LETTER => 11,
      OTHER => 12,
      ACTIVE => 13,
      COMMENT => 14,
      INVALID => 15,
      CS => 16,
      MARKER => 17,
      ARG => 18,
    }
  }
}

impl Catcode {
  /// a debug-friendly name
  pub fn name(self) -> &'static str {
    use crate::token::Catcode::*;
    match self {
      // Primitive
      ESCAPE => "Escape",
      BEGIN => "Begin",
      END => "End",
      MATH => "Math",
      ALIGN => "Align",
      EOL => "EOL",
      PARAM => "Parameter",
      SUPER => "Superscript",
      SUB => "Subscript",
      SPACE => "Space",
      // Non-primitive
      IGNORE => "Ignore",
      LETTER => "Letter",
      OTHER => "Other",
      ACTIVE => "Active",
      COMMENT => "Comment",
      INVALID => "Invalid",
      CS => "ControlSequence",
      MARKER => "Marker",
      ARG => "Arg",
    }
  }

  /// SymStr form of `name()` — each variant caches its interned
  /// symbol via `pin!`. Used by `Token::get_cs_name` /
  /// `pin_cs_name` to avoid a per-call `pin_static` hash probe
  /// (fires on every primitive-token definition lookup).
  pub fn name_sym(self) -> crate::common::arena::SymStr {
    use crate::token::Catcode::*;
    match self {
      ESCAPE => crate::pin!("Escape"),
      BEGIN => crate::pin!("Begin"),
      END => crate::pin!("End"),
      MATH => crate::pin!("Math"),
      ALIGN => crate::pin!("Align"),
      EOL => crate::pin!("EOL"),
      PARAM => crate::pin!("Parameter"),
      SUPER => crate::pin!("Superscript"),
      SUB => crate::pin!("Subscript"),
      SPACE => crate::pin!("Space"),
      IGNORE => crate::pin!("Ignore"),
      LETTER => crate::pin!("Letter"),
      OTHER => crate::pin!("Other"),
      ACTIVE => crate::pin!("Active"),
      COMMENT => crate::pin!("Comment"),
      INVALID => crate::pin!("Invalid"),
      CS => crate::pin!("ControlSequence"),
      MARKER => crate::pin!("Marker"),
      ARG => crate::pin!("Arg"),
    }
  }
  /// a \meaning-friendly name
  pub fn meaning(self) -> &'static str {
    use crate::token::Catcode::*;
    match self {
      ESCAPE => "the escape character",
      BEGIN => "begin-group character",
      END => "end-group character",
      MATH => "math shift character",
      ALIGN => "alignment tab character",
      EOL => "end-of-line character",
      PARAM => "macro parameter character",
      SUPER => "superscript character",
      SUB => "subscript character",
      IGNORE => "ignored character",
      SPACE => "blank space",
      LETTER => "the letter",
      OTHER => "the character",
      ACTIVE => "active character",
      COMMENT => "comment character",
      INVALID => "invalid character",
      _ => "",
    }
  }
  /// a short name helpful for Token debugging
  pub fn short_name(self) -> &'static str {
    use crate::token::Catcode::*;
    match self {
      ESCAPE => "T_ESCAPE",
      BEGIN => "T_BEGIN",
      END => "T_END",
      MATH => "T_MATH",
      ALIGN => "T_ALIGN",
      EOL => "T_EOL",
      PARAM => "T_PARAM",
      SUPER => "T_SUPER",
      SUB => "T_SUB",
      IGNORE => "T_IGNORE",
      SPACE => "T_SPACE",
      LETTER => "T_LETTER",
      OTHER => "T_OTHER",
      ACTIVE => "T_ACTIVE",
      COMMENT => "T_COMMENT",
      INVALID => "T_INVALID",
      CS => "T_CS",
      MARKER => "T_MARKER",
      ARG => "T_ARG",
    }
  }

  // ======================================================================
  // Categories of Category codes.
  // For Tokens with these catcodes, only the catcode is relevant for comparison.
  // (if they even make it to a stage where they get compared)
  /// TeX-primitive codes
  pub fn is_primitive(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Primitives
      ESCAPE | BEGIN | END | MATH | ALIGN | EOL | PARAM | SUPER | SUB | SPACE => true,
      // Non-primitive
      IGNORE | LETTER | OTHER | ACTIVE | COMMENT | INVALID | CS | MARKER | ARG => false,
    }
  }
  /// Catcodes with associated primitives
  pub fn is_executable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Executable
      BEGIN | END | MATH | ALIGN | SUPER | SUB | ACTIVE | CS => true,
      // Non-executable
      EOL | ESCAPE | PARAM | SPACE | IGNORE | LETTER | OTHER | COMMENT | INVALID | MARKER | ARG => {
        false
      },
    }
  }
  /// Catcodes which can be neutralized
  pub fn is_neutralizable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Neutralizable
      MATH | ALIGN | PARAM | SUPER | SUB | ACTIVE => true,
      // Non-neutralizable
      ESCAPE | BEGIN | END | EOL | IGNORE | SPACE | LETTER | OTHER | COMMENT | INVALID | CS
      | MARKER | ARG => false,
    }
  }
  /// Shorthand to match the "active" and "command sequence" catcodes
  pub fn is_active_or_cs(self) -> bool {
    use crate::token::Catcode::*;
    matches!(self, ACTIVE | CS)
  }
  /// Tokens which can be absorbed without side-effects
  pub fn is_absorbable(self) -> bool {
    use crate::token::Catcode::*;
    // Absorbable
    matches!(self, SPACE | LETTER | OTHER | COMMENT)
  }
  /// Gullet can only hold comment and marker tokens
  pub fn is_gullet_holdable(self) -> bool {
    use crate::token::Catcode::*;
    matches!(self, COMMENT | MARKER)
  }
  /// Catcodes that are of note for balanced reads.
  pub fn is_balanced_interesting(self) -> bool {
    use crate::token::Catcode::*;
    // `gullet::is_balanced` reacts to BEGIN,END,MARKER coded tokens
    matches!(self, BEGIN | END | MARKER)
  }
}

/// The core immutable syntactic primitive resulting from TeX's read-in and expansion process
/// We allow the fields to be public, so that we can use builder macros such as
/// ```
/// macro_rules! T_SPACE(() => {
///     Token { text: arena::pin_static(" "), code: Catcode::SPACE}
///   });
/// ```
#[derive(Copy, Clone)]
pub struct Token {
  /// an arena id the character content for this token
  pub text: SymStr,
  /// a TeX catcode
  pub code: Catcode,
  /// Origin handle into the per-conversion token-origin side arena (1-based;
  /// `0` = no recorded origin). Present only under the `token-locators` feature
  /// (the opt-in source-map precision build); `Token` stays 8 bytes otherwise.
  /// Set in `read_token`; carried through expansion so a digested run can recover
  /// its exact source span. **Excluded from `PartialEq`** (tokens compare by
  /// meaning, not origin — see `impl PartialEq`). See docs/SOURCE_PROVENANCE.md
  /// §3.1.1.
  #[cfg(feature = "token-locators")]
  pub loc: u32,
}

impl fmt::Debug for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.code == Catcode::ARG {
      self.with_str(|text| write!(f, "\"#{}\"", text))
    } else {
      self.with_str(|text| write!(f, "{:?}", text))
    }
  }
}

impl Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.code == Catcode::ARG {
      write!(f, "#")?;
    }
    self.with_str(|text| write!(f, "{}", text))
  }
}

/// Compare two tokens; They are equal if they both have same catcode & string
// [We pretend all SPACE's are the same, since we'd like to hide newline's in there!]
// NOTE: That another popular equality checks whether the "meaning" (defn) are the same.
// That is NOT done here; see Equals(x,y) and XEquals(x,y)
impl PartialEq for Token {
  fn eq(&self, other: &Token) -> bool {
    self.code == other.code && (self.code == Catcode::SPACE || (self.text == other.text))
  }
}

// Note: given that we are pinning the strings in an arena,
//  once we have a token of a certain kind it is now faster to clone
//  a known token than it is to build a new one
//  (as the arena lookup is a hair slower than copying a u32)

/// constant for a BEGIN "{" token
#[thread_local]
pub static TOKEN_BEGIN: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("{"),
  code: Catcode::BEGIN,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for an END "}" token
#[thread_local]
pub static TOKEN_END: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("}"),
  code: Catcode::END,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for a MATH "$" token
#[thread_local]
pub static TOKEN_MATH: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("$"),
  code: Catcode::MATH,
      #[cfg(feature = "token-locators")] loc: 0
    });
/// constant for an ALIGN "&" token
#[thread_local]
pub static TOKEN_ALIGN: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("&"),
  code: Catcode::ALIGN,
      #[cfg(feature = "token-locators")] loc: 0
    });
/// constant for a PARAM "#" token
#[thread_local]
pub static TOKEN_PARAM: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("#"),
  code: Catcode::PARAM,
      #[cfg(feature = "token-locators")] loc: 0
    });
/// constant for a SUPER "^" token
#[thread_local]
pub static TOKEN_SUPER: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("^"),
  code: Catcode::SUPER,
      #[cfg(feature = "token-locators")] loc: 0
    });
/// constant for a SUB "_" token
#[thread_local]
pub static TOKEN_SUB: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("_"),
  code: Catcode::SUB,
      #[cfg(feature = "token-locators")] loc: 0
    });
/// constant for a SPACE " " token
#[thread_local]
pub static TOKEN_SPACE: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static(" "),
  code: Catcode::SPACE,
      #[cfg(feature = "token-locators")] loc: 0
    });
/// constant for a CR "\n" token
#[thread_local]
pub static TOKEN_CR: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("\n"),
  code: Catcode::SPACE,
      #[cfg(feature = "token-locators")] loc: 0
    });
/// constant for T_CS("\relax")
#[thread_local]
pub static TOKEN_RELAX: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("\\relax"),
  code: Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    });
/// constant for T_CS("\expandafter")
#[thread_local]
pub static TOKEN_EXPANDAFTER: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("\\expandafter"),
  code: Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    });
/// constant for T_CS("\endcsname")
#[thread_local]
pub static TOKEN_ENDCSNAME: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("\\endcsname"),
  code: Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    });

#[macro_export]
/// macro for a BEGIN "{" token
macro_rules! T_BEGIN(() => { *$crate::token::TOKEN_BEGIN });
#[macro_export]
/// macro for a new END "{" token
macro_rules! T_END(() => { *$crate::token::TOKEN_END });
/// macro for a MATH "$" token
#[macro_export]
macro_rules! T_MATH(() => { *$crate::token::TOKEN_MATH });
/// macro for an ALIGN "&" token
#[macro_export]
macro_rules! T_ALIGN(() => { *$crate::token::TOKEN_ALIGN });
/// macro for a PARAM "#" token
#[macro_export]
macro_rules! T_PARAM(() => { *$crate::token::TOKEN_PARAM });
/// macro for a SUPER "^" token
#[macro_export]
macro_rules! T_SUPER(() => { *$crate::token::TOKEN_SUPER });
/// macro for a SUB "_" token
#[macro_export]
macro_rules! T_SUB(() => { *$crate::token::TOKEN_SUB });
/// macro for a SPACE token (default " ")
#[macro_export]
macro_rules! T_SPACE(() => { *$crate::token::TOKEN_SPACE };
($text:literal) => {
  Token { text: $crate::pin!($text), code: Catcode::SPACE,
      #[cfg(feature = "token-locators")] loc: 0
    }
});
/// macro for a CR "\n" token
#[macro_export]
macro_rules! T_CR(() => { *$crate::token::TOKEN_CR });
/// macro for a LETTER token
#[macro_export]
macro_rules! T_LETTER {
  ($text:literal) => {
    Token {
      text: $crate::pin!($text),
      code: Catcode::LETTER,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
  ($text:expr) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::LETTER,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
}
/// macro for an OTHER code token
#[macro_export]
macro_rules! T_OTHER {
  ($text:literal) => {
    Token {
      text: $crate::pin!($text),
      code: Catcode::OTHER,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
  ($text:expr) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::OTHER,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
}
/// T_OTHER from a single character
#[macro_export]
macro_rules! T_OTHER_CHAR {
  ($text:literal) => {
    Token {
      text: $crate::common::arena::pin_char($text),
      code: Catcode::OTHER,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
}
/// macro for an ACTIVE char token
#[macro_export]
macro_rules! T_ACTIVE {
  ($c:expr) => {{
    let mut tmp = [0u8; 4];
    let s = $c.encode_utf8(&mut tmp);
    Token {
      text: $crate::common::arena::pin(s),
      code: Catcode::ACTIVE,
      #[cfg(feature = "token-locators")] loc: 0
    }
  }};
}
/// macro for a COMMENT content token
#[macro_export]
macro_rules! T_COMMENT {
  ($text:expr) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::COMMENT,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
}
/// macro for a command sequence token
#[macro_export]
macro_rules! T_CS {
  ($text:literal) => {
    $crate::token::Token {
      text: $crate::pin!($text),
      code: $crate::token::Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
  ($text:expr) => {
    $crate::token::Token {
      text: $crate::common::arena::pin($text),
      code: $crate::token::Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
}

/// macro for T_CS("\\relax")
#[macro_export]
macro_rules! T_RELAX(() => { $crate::token::TOKEN_RELAX.clone() });

/// macro for a tracing MARKER token
#[macro_export]
macro_rules! T_MARKER {
  ($text:expr) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::MARKER,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
}

/// macro for a numbered ARG token
#[macro_export]
macro_rules! T_ARG {
  ($text:expr) => {
    Token {
      text: $crate::common::arena::pin($text.to_string()),
      code: Catcode::ARG,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
}

/// Token constructor macro (defaults to OTHER code)
#[macro_export]
macro_rules! Token {
  ($text:expr) => {
    Token!($text, Catcode::OTHER)
  };
  ($text:literal, $cc:expr) => {
    Token {
      text: $crate::pin!($text),
      code: $cc,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
  ($text:expr, $cc:expr) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: $cc,
      #[cfg(feature = "token-locators")] loc: 0
    }
  };
}

/// Special case: a character needs swift string conversion, so let's use a dedicated macro
#[macro_export]
macro_rules! CharToken {
  ($c:expr) => {
    CharToken!($c, Catcode::OTHER)
  };
  ($c:expr, $cc:expr) => {{
    let mut tmp = [0u8; 4];
    let s = $c.encode_utf8(&mut tmp);
    Token!(s, $cc)
  }};
}

/// Explode a string into a list of tokens, all w/catcode OTHER (except space).
/// Note: newlines are converted to OTHER, NOT SPACE (Perl #2700 reverted #2646).
/// ^^J in TeX decodes to CC_OTHER by default; let the tokenizer handle catcode
/// reassignment if needed.
#[macro_export]
macro_rules! Explode(($text:expr) => (
  $text.to_string().chars().map(|c|
    if c==' ' { T_SPACE!() }
    else {
      CharToken!(c)
    }
  ).collect::<Vec<Token>>()
));

#[macro_export]
macro_rules! ExplodeChars(($text:expr) => (
  $text.as_str().chars().map(|c|
    if c==' ' { T_SPACE!() }
    else {
      CharToken!(c)
    }
  ).collect::<Vec<Token>>()
));

/// Similar to Explode, but convert letters to catcode LETTER and others to OTHER
/// Hopefully, this is essentially correct WITHOUT resorting to catcode lookup?
/// Perl sync: newlines are OTHER, not SPACE (matches Perl #2700 revert of #2646).
#[macro_export]
macro_rules! ExplodeText(
  ($text:expr) => ({
  use $crate::token::{Catcode,Token};
  $text.to_string().chars().map(|c|
    if c==' ' { T_SPACE!() }
    else {
      let mut tmp = [0u8; 4];
      let s = c.encode_utf8(&mut tmp);
      if c.is_alphabetic() {
      T_LETTER!(s) }
    else { T_OTHER!(s) }}
  ).collect::<Vec<Token>>()
}));

#[macro_export]
macro_rules! SymExplodeText(
  ($sym:expr) => ({
  use $crate::token::{Catcode,Token};
  let chars : Vec<char> = arena::with($sym, |text| text.chars().collect());
  chars.into_iter().map(|c|
    if c==' ' { T_SPACE!() }
    else {
      let mut tmp = [0u8; 4];
      let s = c.encode_utf8(&mut tmp);
      if c.is_alphabetic() {
      T_LETTER!(s) }
    else { T_OTHER!(s) }}
  ).collect::<Vec<Token>>()
}));

// static UNTEX_LINELENGTH: usize = 78; // [CONSTANT]

impl Default for Token {
  fn default() -> Self {
    Token {
      text: arena::pin_static("EXPECTED_TOKEN"),
      code: Catcode::OTHER,
      #[cfg(feature = "token-locators")] loc: 0
    }
  }
}

///======================================================================
/// Accessors.
impl Token {
  /// simple Token constructor, wrapping over text and catcode
  pub fn new<T: AsRef<str>>(text: T, code: Catcode) -> Self {
    Token { text: arena::pin(text), code, #[cfg(feature = "token-locators")] loc: 0 }
  }

  /// Get the CS Name of the token. This is the name that definitions will be
  /// stored under; It's the same for various `different' BEGIN tokens, eg.
  pub fn get_cs_name(&self) -> SymStr {
    if self.code.is_primitive() {
      self.code.name_sym()
    } else {
      self.get_sym()
    }
  }

  /// Execute a closure using the CS Name of the token.
  /// This is the name that definitions will be stored under;
  /// It's the same for various `different' BEGIN tokens, eg.
  pub fn with_cs_name<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&str) -> R {
    if self.code.is_primitive() {
      caller(self.code.name())
    } else {
      self.with_str(caller)
    }
  }

  /// artificial, but avoids the data race
  pub fn pin_cs_name(&self) -> SymStr {
    if self.code.is_primitive() {
      self.code.name_sym()
    } else {
      self.get_sym()
    }
  }

  /// Get the fixed name of a primitive catcode, or empty string otherwise
  pub fn get_primitive_name(&self) -> Option<&'static str> {
    if self.code.is_primitive() {
      Some(self.code.name())
    } else {
      None
    }
  }

  /// Get the CS name only if the catcode is executable!
  pub fn get_executable_name(&self) -> String {
    let cc = self.code;
    if cc.is_executable() {
      self
        .get_primitive_name()
        .map(ToString::to_string)
        .unwrap_or_else(|| self.with_str(|text| text.to_string()))
    } else {
      String::new()
    }
  }

  /// Intersect executable and primitive
  pub fn get_executable_primitive_name(&self) -> Option<&'static str> {
    let cc = self.code;
    if cc.is_executable() && cc.is_primitive() {
      Some(self.code.name())
    } else {
      None
    }
  }

  /// Use the ticket representing the interned "text" of the token
  pub fn get_sym(&self) -> SymStr { self.text }
  /// Use the interned &str "text" of the token
  /// use `to_string` instead for an owned String with simpler
  pub fn with_str<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&str) -> R {
    arena::with(self.text, caller)
  }

  /// Return the character code of  character part of the token, or 256 if it is a control
  /// sequence
  pub fn get_charcode(&self) -> u32 {
    if self.code == Catcode::CS {
      256
    } else {
      self.with_str(|text| {
        if let Some(c) = text.chars().next() {
          c as u32
        } else {
          0
        }
      })
    }
  }

  /// Return the catcode of the token.
  pub fn get_catcode(&self) -> Catcode { self.code }
  /// is the current one
  pub fn is_executable(&self) -> bool { self.code.is_executable() }

  /// neutralize really should only retroactively imitate what Semiverbatim would have done.
  /// So, it needs to neutralize those in SPECIALS
  /// NOTE that although '%' gets it's catcode changed in Semiverbatim,
  /// I'm pretty sure we do NOT want to neutralize comments (turn them into Catcode::OTHER)
  /// here, since if comments do get into the Tokens, that will introduce weird crap into the
  /// stream.
  pub fn neutralize(self, extraspecials: &[char]) -> Token {
    let first_c: Option<char> = self.with_str(|text| text.chars().next());
    let ch = match first_c {
      Some(ch) => ch,
      None => return self,
    };
    let cc = self.code;
    if cc.is_neutralizable() {
      for extra in extraspecials {
        if extra == &ch {
          let mut tmp = [0u8; 4];
          let s = ch.encode_utf8(&mut tmp);
          return T_OTHER!(s);
        }
      }
      let maybe_return = state::with_value("SPECIALS", |specials_opt| {
        if let Some(Stored::Chars(ref specials_list)) = specials_opt {
          for special in specials_list.iter() {
            if *special == ch {
              let mut tmp = [0u8; 4];
              let s = ch.encode_utf8(&mut tmp);
              return Some(T_OTHER!(s));
            }
          }
        }
        None
      });
      if let Some(token) = maybe_return {
        return token;
      }
    }
    self
  }

  pub fn as_other(&self) -> Token {
    Token {
      text: self.text,
      code: Catcode::OTHER,
      #[cfg(feature = "token-locators")] loc: 0
    }
  }
  pub fn as_cs(&self) -> Token {
    Token {
      text: self.text,
      code: Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    }
  }

  pub fn substitute_parameters(self, args: &[&Token]) -> Self {
    if self.code == Catcode::ARG {
      self.with_str(|text| {
        let arg_idx = text
          .parse::<usize>()
          .expect("ARG catcode tokens should always contain numeric literals as text");
        *args[arg_idx - 1]
      })
    } else {
      self
    }
  }

  /// A Token reverts to itself
  pub fn revert(self) -> Token { self }

  /// A string form which is primarily used for error-reporting
  pub fn stringify(&self) -> String {
    self.with_str(|text| {
      // Make the token's char content more printable, since this is for a visual messages.
      let display_text = if text.len() == 1 {
        let c = text.chars().next().unwrap() as u16;
        if c < 0x020 {
          Cow::Owned(s!("U+{:04x}/{}", c, CONTROLNAME[c as usize]))
        } else {
          Cow::Borrowed(text)
        }
      } else {
        Cow::Borrowed(text)
      };
      s!("{}[{}]", self.code.short_name(), display_text)
    })
  }

  pub fn to_register(&self) -> Option<Rc<Register>> { state::lookup_register_definition(self) }

  pub fn to_number(&self) -> Number {
    Number::new(self.with_str(|text| text.parse::<i64>()).unwrap_or(0))
  }

  pub fn to_dimension(&self) -> Dimension {
    Dimension::new_f64(self.with_str(|text| text.parse::<f64>().unwrap_or(0.0)))
  }

  pub fn to_mu_dimension(&self) -> MuDimension {
    MuDimension::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn to_glue(&self) -> Glue {
    Glue::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn to_mu_glue(&self) -> MuGlue {
    MuGlue::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn to_float(&self) -> Float {
    Float::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn be_digested(self) -> Result<Digested> { crate::stomach::digest(Tokens::new(vec![self])) }

  /// Check whether the current token is defined as `other`.
  /// That is, whether it is equal to `other`, or \let to `other`.
  /// `other` is is presumed to be some "constant", explicit token,
  /// such as  `T_SPACE` or `T_CS!("\\endcsname")`.
  pub fn defined_as(&self, other: &Token) -> bool {
    let cc = self.code;
    let occ = other.get_catcode();
    if (cc == occ) && ((occ == Catcode::SPACE) || (self.text == other.get_sym())) {
      return true;
    }
    if matches!(cc, Catcode::CS | Catcode::ACTIVE) {
      // Use the closure-based `with_meaning` — Token is `Copy`, so
      // extracting a `Token` from the borrowed Stored is an implicit
      // copy, not a clone. Avoids a full `Stored::clone()` per call
      // (defined_as fires ~1% of total instructions on siunitx/
      // physics-heavy docs).
      let letto_opt: Option<Token> = state::with_meaning(self, |defn_opt| {
        defn_opt.and_then(|defn| match defn {
          Stored::Token(t) => Some(*t),
          Stored::Expandable(inner) => Some(*inner.get_cs()),
          Stored::Primitive(inner) => Some(*inner.get_cs()),
          Stored::MathPrimitive(inner) => Some(*inner.get_cs()),
          Stored::Register(inner) => Some(*inner.get_cs()),
          Stored::Conditional(inner) => Some(*inner.get_cs()),
          Stored::Constructor(inner) => Some(*inner.get_cs()),
          _ => None,
        })
      });
      if let Some(letto) = letto_opt {
        if (letto.get_catcode() == occ)
          && ((occ == Catcode::SPACE) || letto.get_sym() == other.get_sym())
        {
          return true;
        }
      }
    }
    false
  }
}

// A simple (constant!) auto-cast for &str to Token. Beware this will not respect the current
// catcodes in state (and @ is OTHER).
impl From<&str> for Token {
  fn from(text: &str) -> Token {
    match text.chars().next() {
      Some('{') => T_BEGIN!(),
      Some('}') => T_END!(),
      Some('$') => T_MATH!(),
      Some('#') => T_PARAM!(),
      Some('&') => T_ALIGN!(),
      Some('^') => T_SUPER!(),
      Some('_') => T_SUB!(),
      Some('\\') => T_CS!(text),
      Some('%') => T_COMMENT!(text),
      _ => {
        if text.chars().all(|c| c.is_alphabetic()) {
          T_LETTER!(text)
        } else if text.chars().all(|c| c.is_whitespace()) {
          T_SPACE!()
        } else {
          T_OTHER!(text)
        }
      },
    }
  }
}

// ── Token-origin side arena (token-locators feature) ───────────────────────
// Per-conversion store mapping a Token's `loc` handle (1-based; 0 = none) to its
// captured source start. Tokens carry only the u32 handle (Token stays 12 bytes);
// this holds the (source, line, col). Appended in `read_token`, read by the
// digestion consumer to give a text run its true span. Cleared per conversion.
// See docs/SOURCE_PROVENANCE.md §3.1.1.
#[cfg(feature = "token-locators")]
#[derive(Clone, Copy, Debug)]
pub struct TokenStart {
  pub source: SymStr,
  pub line:   u32,
  pub col:    u32,
}

#[cfg(feature = "token-locators")]
thread_local! {
  static TOKEN_ORIGINS: std::cell::RefCell<Vec<TokenStart>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Append a token's source start, returning its 1-based handle (`0` is reserved
/// for "no origin"). Only called on the source-map precision path.
#[cfg(feature = "token-locators")]
pub fn push_token_origin(source: SymStr, line: u32, col: u32) -> u32 {
  TOKEN_ORIGINS.with(|o| {
    let mut v = o.borrow_mut();
    v.push(TokenStart { source, line, col });
    v.len() as u32 // index + 1
  })
}

/// Resolve a token `loc` handle to its origin (`None` for the `0` sentinel or an
/// out-of-range handle).
#[cfg(feature = "token-locators")]
pub fn get_token_origin(handle: u32) -> Option<TokenStart> {
  if handle == 0 {
    return None;
  }
  TOKEN_ORIGINS.with(|o| o.borrow().get((handle - 1) as usize).copied())
}

/// Reset the arena at the start of a conversion (handles are per-conversion).
#[cfg(feature = "token-locators")]
pub fn clear_token_origins() {
  TOKEN_ORIGINS.with(|o| o.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
  use super::*;

  /// `Token` size invariant (docs/SOURCE_PROVENANCE.md §3.1.1): 8 bytes by
  /// default (`SymStr` + `Catcode`), 12 only under the `token-locators`
  /// precision build (+ the `u32` origin handle). Guards the corpus/parity/
  /// distribution builds against an accidental widening.
  #[test]
  fn token_size_invariant() {
    #[cfg(not(feature = "token-locators"))]
    assert_eq!(
      std::mem::size_of::<Token>(),
      8,
      "default Token must stay 8 bytes (SymStr + Catcode)"
    );
    #[cfg(feature = "token-locators")]
    assert_eq!(
      std::mem::size_of::<Token>(),
      12,
      "token-locators Token is 8 + a u32 origin handle"
    );
  }

  /// Per-token origin capture (token-locators): each char token read from a
  /// mouth carries a handle resolving to its exact (line, col). This is the leaf
  /// accuracy that mouth-snapshot (Experiments 1–2) and digested-child assembly
  /// (Experiment 3) could not provide — the position now travels *with the
  /// token*. See docs/SOURCE_PROVENANCE.md §3.1.1.
  #[cfg(feature = "token-locators")]
  #[test]
  fn token_origin_capture() {
    super::clear_token_origins();
    // "Hello" — five letters at 1-indexed columns 1..=5 on line 1.
    let toks = crate::mouth::tokenize("Hello");
    let got: Vec<(u32, u32)> = toks
      .unlist_ref()
      .iter()
      .map(|t| {
        let o = super::get_token_origin(t.loc).expect("token carries an origin handle");
        (o.line, o.col)
      })
      .collect();
    assert_eq!(
      got,
      vec![(1, 1), (1, 2), (1, 3), (1, 4), (1, 5)],
      "each letter's captured (line, col) must be exact"
    );
  }

  #[test]
  fn catcode_name_covers_all_variants() {
    // Ensure every Catcode variant produces a non-empty name.
    use Catcode::*;
    for cc in [
      ESCAPE, BEGIN, END, MATH, ALIGN, EOL, PARAM, SUPER, SUB, SPACE, IGNORE, LETTER, OTHER,
      ACTIVE, COMMENT, INVALID, CS, MARKER, ARG,
    ] {
      assert!(!cc.name().is_empty(), "{cc:?}.name() is empty");
    }
  }

  #[test]
  fn catcode_name_specific_values() {
    assert_eq!(Catcode::ESCAPE.name(), "Escape");
    assert_eq!(Catcode::BEGIN.name(), "Begin");
    assert_eq!(Catcode::CS.name(), "ControlSequence");
    assert_eq!(Catcode::LETTER.name(), "Letter");
  }

  #[test]
  fn catcode_short_name_starts_with_t_prefix() {
    use Catcode::*;
    for cc in [
      ESCAPE, BEGIN, END, MATH, ALIGN, EOL, PARAM, SUPER, SUB, SPACE, IGNORE, LETTER, OTHER,
      ACTIVE, COMMENT, INVALID, CS, MARKER, ARG,
    ] {
      assert!(
        cc.short_name().starts_with("T_"),
        "{cc:?}.short_name() = {} lacks T_ prefix",
        cc.short_name()
      );
    }
  }

  #[test]
  fn catcode_meaning_mostly_nonempty() {
    // Most variants have a TeX-meaning description.
    assert!(!Catcode::ESCAPE.meaning().is_empty());
    assert!(!Catcode::LETTER.meaning().is_empty());
    assert!(!Catcode::OTHER.meaning().is_empty());
    // A few (e.g. CS/MARKER/ARG) fall through to "".
    assert_eq!(Catcode::CS.meaning(), "");
  }

  #[test]
  fn is_primitive_checks() {
    use Catcode::*;
    // TeX primitives:
    for cc in [
      ESCAPE, BEGIN, END, MATH, ALIGN, EOL, PARAM, SUPER, SUB, SPACE,
    ] {
      assert!(cc.is_primitive(), "{cc:?} should be primitive");
    }
    // Non-primitives:
    for cc in [
      IGNORE, LETTER, OTHER, ACTIVE, COMMENT, INVALID, CS, MARKER, ARG,
    ] {
      assert!(!cc.is_primitive(), "{cc:?} should not be primitive");
    }
  }

  #[test]
  fn is_executable_checks() {
    use Catcode::*;
    for cc in [BEGIN, END, MATH, ALIGN, SUPER, SUB, ACTIVE, CS] {
      assert!(cc.is_executable(), "{cc:?} should be executable");
    }
    for cc in [
      EOL, ESCAPE, PARAM, SPACE, IGNORE, LETTER, OTHER, COMMENT, INVALID, MARKER, ARG,
    ] {
      assert!(!cc.is_executable(), "{cc:?} should not be executable");
    }
  }

  #[test]
  fn is_neutralizable_set() {
    use Catcode::*;
    for cc in [MATH, ALIGN, PARAM, SUPER, SUB, ACTIVE] {
      assert!(cc.is_neutralizable(), "{cc:?}");
    }
    assert!(!Catcode::CS.is_neutralizable());
    assert!(!Catcode::LETTER.is_neutralizable());
  }

  #[test]
  fn is_active_or_cs_narrow_set() {
    assert!(Catcode::ACTIVE.is_active_or_cs());
    assert!(Catcode::CS.is_active_or_cs());
    assert!(!Catcode::LETTER.is_active_or_cs());
    assert!(!Catcode::ESCAPE.is_active_or_cs());
  }

  #[test]
  fn is_absorbable_space_letter_other_comment() {
    use Catcode::*;
    assert!(SPACE.is_absorbable());
    assert!(LETTER.is_absorbable());
    assert!(OTHER.is_absorbable());
    assert!(COMMENT.is_absorbable());
    // All else not absorbable.
    assert!(!Catcode::CS.is_absorbable());
    assert!(!Catcode::BEGIN.is_absorbable());
  }

  #[test]
  fn is_gullet_holdable_comment_marker_only() {
    assert!(Catcode::COMMENT.is_gullet_holdable());
    assert!(Catcode::MARKER.is_gullet_holdable());
    assert!(!Catcode::SPACE.is_gullet_holdable());
    assert!(!Catcode::LETTER.is_gullet_holdable());
  }

  #[test]
  fn is_balanced_interesting_begin_end_marker() {
    assert!(Catcode::BEGIN.is_balanced_interesting());
    assert!(Catcode::END.is_balanced_interesting());
    assert!(Catcode::MARKER.is_balanced_interesting());
    assert!(!Catcode::LETTER.is_balanced_interesting());
    assert!(!Catcode::MATH.is_balanced_interesting());
  }

  #[test]
  fn catcode_u8_roundtrip() {
    // From<Catcode> for u8 + From<u8> for Catcode should round-trip
    // (at least for the documented range 0..=18).
    use Catcode::*;
    for cc in [
      ESCAPE, BEGIN, END, MATH, ALIGN, EOL, PARAM, SUPER, SUB, SPACE, IGNORE, LETTER, OTHER,
      ACTIVE, COMMENT, INVALID, CS, MARKER, ARG,
    ] {
      let b: u8 = cc.into();
      let cc2: Catcode = b.into();
      assert_eq!(cc, cc2, "roundtrip broke for {cc:?} (u8={b})");
    }
  }

  #[test]
  fn token_new_and_display() {
    let t = Token::new("foo", Catcode::LETTER);
    assert_eq!(format!("{t}"), "foo");
    assert_eq!(t.code, Catcode::LETTER);
  }

  #[test]
  fn token_arg_display_prepends_hash() {
    // ARG catcode prepends # in Display.
    let t = Token::new("1", Catcode::ARG);
    assert_eq!(format!("{t}"), "#1");
  }
}
