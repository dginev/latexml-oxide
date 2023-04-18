use std::fmt;
use std::fmt::Display;
use std::sync::Arc;
use string_interner::symbol::SymbolU32;

use crate::common::arena;
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::float::Float;
use crate::common::glue::Glue;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::common::store::Stored;
use crate::definition::register::Register;
use crate::state::State;
use crate::stomach::Stomach;
use crate::tokens::Tokens;
use crate::{fatal, Digested, Fatal};

static CONTROLNAME: &[&str] = &[
  "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL", "BS", "HT", "LF", "VT", "FF", "CR", "SO",
  "SI", "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN", "EM", "SUB", "ESC", "FS",
  "GS", "RS", "US",
];

/// A Token category code, as in TeX
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Catcode {
  ///
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
  SmuggleTHE,
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
      19 => SmuggleTHE,
      _ => {
        // let message = s!("Unrecognized catcode: {:?}", num);
        // Warn!("unknown", "catcode", None, None, message);
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
      SmuggleTHE => 19,
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
      SmuggleTHE => "SmuggleThe",
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
      SmuggleTHE => "T_SMUGGLE_THE",
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
      IGNORE | LETTER | OTHER | ACTIVE | COMMENT | INVALID | CS | MARKER | ARG | SmuggleTHE => {
        false
      },
    }
  }
  ///
  pub fn is_executable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Executable
      BEGIN | END | MATH | ALIGN | SUPER | SUB | ACTIVE | CS => true,
      // Non-executable
      EOL | ESCAPE | PARAM | SPACE | IGNORE | LETTER | OTHER | COMMENT | INVALID | MARKER | ARG
      | SmuggleTHE => false,
    }
  }
  ///
  pub fn is_neutralizable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Neutralizable
      MATH | ALIGN | PARAM | SUPER | SUB | ACTIVE => true,
      // Non-neutralizable
      ESCAPE | BEGIN | END | EOL | IGNORE | SPACE | LETTER | OTHER | COMMENT | INVALID | CS
      | MARKER | ARG | SmuggleTHE => false,
    }
  }
  ///
  pub fn is_active_or_cs(self) -> bool {
    use crate::token::Catcode::*;
    matches!(self, ACTIVE | CS)
  }
  ///
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
  /// gullet::is_balanced reacts to BEGIN,END,MARKER coded tokens
  pub fn is_balanced_interesting(self) -> bool {
    use crate::token::Catcode::*;
    matches!(self, BEGIN | END | MARKER)
  }
  /// can a token of this catcode be used to smuggle for `\the`
  pub fn can_smuggle_the(self) -> bool {
    use crate::token::Catcode::*;
    matches!(self, PARAM | ACTIVE | CS | ARG)
  }
}

/// The core immutable syntactic primitive resulting from TeX's read-in and expansion process
/// We allow the fields to be public, so that we can use builder macros such as
/// ```
/// macro_rules! T_SPACE(() => {
///     Token { text: arena::pin(" "), code: Catcode::SPACE}
///   });
/// ```
#[derive(Clone)]
pub struct Token {
  /// an arena id the character content for this token
  pub text: SymbolU32,
  /// a TeX catcode
  pub code: Catcode,
  /// possibly smuggled inner token (for \noexpand)
  pub smuggled: Option<Box<Token>>,
}

impl fmt::Debug for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.code == Catcode::ARG {
      self.with_str(|text|
        write!(f, "\"#{}\"", text))
    } else {
      self.with_str(|text|
        write!(f, "{:?}", text)
      )
    }
  }
}

impl Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.code == Catcode::ARG {
      write!(f, "#")?;
    }
    self.with_str(|text|
      write!(f, "{}", text)
    )
  }
}

/// Compare two tokens; They are equal if they both have same catcode & string
// [We pretend all SPACE's are the same, since we'd like to hide newline's in there!]
// NOTE: That another popular equality checks whether the "meaning" (defn) are the same.
// That is NOT done here; see Equals(x,y) and XEquals(x,y)
impl PartialEq for Token {
  fn eq(&self, other: &Token) -> bool {
    self.code == other.code
      && (self.code == Catcode::SPACE || (self.text == other.text))
      && (self.smuggled.is_none() == other.smuggled.is_none())
  }
}

// Note: given that we are pinning the strings in an arena,
//  once we have a token of a certain kind it is now faster to clone
//  a known token than it is to build a new one
//  (as the arena lookup is a hair slower than copying a u32)

thread_local! {
  /// constant for an END "}" token
  pub static TOKEN_BEGIN: Token = Token {
    text: arena::pin("{"),
    code: Catcode::BEGIN,
    smuggled: None,
  };
  /// constant for a BEGIN "{" token
  pub static TOKEN_END: Token = Token {
    text: arena::pin("}"),
    code: Catcode::END,
    smuggled: None,
  };
  /// constant for a MATH "$" token
  pub static TOKEN_MATH: Token = Token {
    text: arena::pin("$"),
    code: Catcode::MATH,
    smuggled: None,
  };
  /// constant for an ALIGN "&" token
  pub static TOKEN_ALIGN: Token = Token {
    text: arena::pin("&"),
    code: Catcode::ALIGN,
    smuggled: None,
  };
  /// constant for a PARAM "#" token
  pub static TOKEN_PARAM: Token = Token {
    text: arena::pin("#"),
    code: Catcode::PARAM,
    smuggled: None,
  };
  /// constant for a SUPER "^" token
  pub static TOKEN_SUPER: Token = Token {
    text: arena::pin("^"),
    code: Catcode::SUPER,
    smuggled: None,
  };
  /// constant for a SUB "_" token
  pub static TOKEN_SUB: Token = Token {
    text: arena::pin("_"),
    code: Catcode::SUB,
    smuggled: None,
  };
  /// constant for a SPACE " " token
  pub static TOKEN_SPACE: Token = Token {
    text: arena::pin(" "),
    code: Catcode::SPACE,
    smuggled: None,
  };
  /// constant for a CR "\n" token
  pub static TOKEN_CR: Token = Token {
    text: arena::pin("\n"),
    code: Catcode::SPACE,
    smuggled: None,
  };
  /// constant for T_CS("\relax")
  pub static TOKEN_RELAX: Token = Token {
    text: arena::pin("\\relax"),
    code: Catcode::CS,
    smuggled: None,
  };
}

#[macro_export]
/// macro for a BEGIN "{" token
macro_rules! T_BEGIN(() => { $crate::token::TOKEN_BEGIN.with(|t| t.clone()) });
#[macro_export]
/// macro for a new END "{" token
macro_rules! T_END(() => { $crate::token::TOKEN_END.with(|t| t.clone()) });
/// macro for a MATH "$" token
#[macro_export]
macro_rules! T_MATH(() => { $crate::token::TOKEN_MATH.with(|t| t.clone()) });
/// macro for an ALIGN "&" token
#[macro_export]
macro_rules! T_ALIGN(() => { $crate::token::TOKEN_ALIGN.with(|t| t.clone()) });
/// macro for a PARAM "#" token
#[macro_export]
macro_rules! T_PARAM(() => { $crate::token::TOKEN_PARAM.with(|t| t.clone()) });
/// macro for a SUPER "^" token
#[macro_export]
macro_rules! T_SUPER(() => { $crate::token::TOKEN_SUPER.with(|t| t.clone()) });
/// macro for a SUB "_" token
#[macro_export]
macro_rules! T_SUB(() => { $crate::token::TOKEN_SUB.with(|t| t.clone()) });
/// macro for a SPACE token (default " ")
#[macro_export]
macro_rules! T_SPACE(() => { $crate::token::TOKEN_SPACE.with(|t| t.clone()) };
($text:literal) => {
  Token { text: $crate::common::arena::pin($text), code: Catcode::SPACE, smuggled: None}
});
/// macro for a CR "\n" token
#[macro_export]
macro_rules! T_CR(() => { $crate::token::TOKEN_CR.with(|t| t.clone()) });
/// macro for a LETTER token
#[macro_export]
macro_rules! T_LETTER {
  ($text:literal) => {
    Token {
      text: $crate::common::arena::pin_static($text),
      code: Catcode::LETTER,
      smuggled: None,
    }
  };
  ($text:expr) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::LETTER,
      smuggled: None,
    }
  };
}
/// macro for an OTHER code token
#[macro_export]
macro_rules! T_OTHER {
  ($text:literal) => {
    Token {
      text: $crate::common::arena::pin_static($text),
      code: Catcode::OTHER,
      smuggled: None,
    }
  };
  ($text:expr) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::OTHER,
      smuggled: None,
    }
  };
}
/// macro for an ACTIVE char token
#[macro_export]
macro_rules! T_ACTIVE {
  ($c:expr) => {{
    let mut tmp = [0u8; 3];
    let s = $c.encode_utf8(&mut tmp);
    Token {
      text: $crate::common::arena::pin(s),
      code: Catcode::ACTIVE,
      smuggled: None,
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
      smuggled: None,
    }
  };
}
/// macro for a command sequence token
#[macro_export]
macro_rules! T_CS {
  ($text:literal) => {
    $crate::token::Token {
      text: $crate::common::arena::pin_static($text),
      code: $crate::token::Catcode::CS,
      smuggled: None,
    }
  };
  ($text:expr) => {
    $crate::token::Token {
      text: $crate::common::arena::pin($text),
      code: $crate::token::Catcode::CS,
      smuggled: None,
    }
  };
}

/// macro for T_CS("\\relax")
#[macro_export]
macro_rules! T_RELAX(() => { $crate::token::TOKEN_RELAX.with(|t| t.clone()) });

/// macro for a tracing MARKER token
#[macro_export]
macro_rules! T_MARKER {
  ($text:expr) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::MARKER,
      smuggled: None,
    }
  };
}

/// macro for a numbered ARG token
#[macro_export]
macro_rules! T_ARG {
  ($text:expr) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::ARG,
      smuggled: None,
    }
  };
}
/// macro for a SmuggleThe token (see `gullet::invoke_and_read_x_token`)
#[macro_export]
macro_rules! T_SMUGGLE_THE {
  ($t:ident) => {
    match $t.get_catcode() {
      Catcode::SmuggleTHE => {
        // LaTeXML Bug, we haven't correctly emulated scan_toks! Offending token was:
        fatal!(
          SmuggledCatcode,
          Unexpected,
          s!(
            "We are masking a \\the-produced token twice, this must Never happen. Illegal: {}",
            $t.stringify()
          )
        );
      },
      cc if cc.can_smuggle_the() => Token {
        text: $crate::common::arena::pin("SMUGGLE_THE"),
        code: Catcode::SmuggleTHE,
        smuggled: Some(Box::new($t)),
      },
      _ => $t,
    }
  };
}

/// Token constructor macro (defaults to OTHER code)
#[macro_export]
macro_rules! Token {
  ($text:expr) => {
    Token!($text, Catcode::OTHER)
  };
  ($text:expr, $cc:expr) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: $cc,
      smuggled: None,
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
    let mut tmp = [0u8; 3];
    let s = $c.encode_utf8(&mut tmp);
    Token!(s, $cc)
  }};
}

/// Explode a string into a list of tokens, all w/catcode OTHER (except space).
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
#[macro_export]
macro_rules! ExplodeText(($text:expr) => ({
  use $crate::token::{Catcode,Token};
  $text.to_string().chars().map(|c|
    if c==' ' { T_SPACE!() }
    else {
      let mut tmp = [0u8; 3];
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
      text: arena::pin("EXPECTED_TOKEN"),
      code: Catcode::OTHER,
      smuggled: None,
    }
  }
}

///======================================================================
/// Accessors.
impl Token {
  /// simple Token constructor, wrapping over text and catcode
  pub fn new<T: AsRef<str>>(text: T, code: Catcode) -> Self {
    Token {
      text: arena::pin(text),
      code,
      smuggled: None,
    }
  }

  /// Get the CS Name of the token. This is the name that definitions will be
  /// stored under; It's the same for various `different' BEGIN tokens, eg.
  pub fn with_cs_name<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&str) -> R {
    if self.code.is_primitive() {
      caller(self.code.name())
    } else {
      self.with_str(caller)
    }
  }

  /// artificial, but avoids the data race
  pub fn pin_cs_name(&self) -> SymbolU32 {
    if self.code.is_primitive() {
      arena::pin_static(self.code.name())
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
        .get_primitive_name().map(ToString::to_string)
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
  pub fn get_sym(&self) -> SymbolU32 {
    self.text
  }
  /// Use the interned &str "text" of the token
  /// use `to_string` instead for an owned String with simpler
  pub fn with_str<R,FnR>(&self, caller: FnR) -> R
    where FnR: FnOnce(&str) -> R {
    arena::with(self.text, caller)
  }

  /// Return the character code of  character part of the token, or 256 if it is a control
  /// sequence
  pub fn get_charcode(&self) -> u32 {
    if self.code == Catcode::CS {
      256
    } else {
      self.with_str(|text|
        if let Some(c) = text.chars().next() {
          c as u32
        } else {
          0
        }
      )
    }
  }

  /// Return the catcode of the token.
  pub fn get_catcode(&self) -> Catcode { self.code }
  /// is the current one
  pub fn is_executable(&self) -> bool { self.code.is_executable() }
  pub fn has_smuggled(&self) -> bool { self.smuggled.is_some() }

  /// neutralize really should only retroactively imitate what Semiverbatim would have done.
  /// So, it needs to neutralize those in SPECIALS
  /// NOTE that although '%' gets it's catcode changed in Semiverbatim,
  /// I'm pretty sure we do NOT want to neutralize comments (turn them into CC_OTHER)
  /// here, since if comments do get into the Tokens, that will introduce weird crap into the
  /// stream.
  pub fn neutralize(self, extraspecials: &[char], state: &State) -> Token {
    let first_c : Option<char> = self.with_str(|text| text.chars().next());
    let ch = match first_c {
      Some(ch) => ch,
      None => return self,
    };
    let cc = self.code;
    if cc.is_neutralizable() {
      for extra in extraspecials {
        if extra == &ch {
          let mut tmp = [0u8; 3];
          let s = ch.encode_utf8(&mut tmp);
          return T_OTHER!(s);
        }
      }
      if let Some(Stored::VecChar(ref specials_list)) = state.lookup_value("SPECIALS") {
        for special in specials_list.iter() {
          if *special == ch {
            let mut tmp = [0u8; 3];
            let s = ch.encode_utf8(&mut tmp);
            return T_OTHER!(s);
          }
        }
      }
    }
    self
  }

  pub fn as_other(&self) -> Token {
    Token {
      text: self.text,
      code: Catcode::OTHER,
      smuggled: None
    }
  }
  pub fn as_cs(&self) -> Token {
    Token {
      text: self.text,
      code: Catcode::CS,
      smuggled: None
    }
  }

  pub fn substitute_parameters(self, args: &[&Token]) -> Self {
    if self.code == Catcode::ARG {
      self.with_str(|text| {
        let arg_idx = text
          .parse::<usize>()
          .expect("ARG catcode tokens should always contain numeric literals as text");
        args[arg_idx - 1].clone()
      })
    } else {
      self
    }
  }

  pub fn pack_parameters(self) -> Self { self }

  pub fn with_dont_expand(self, state: &State) -> Result<Self> {
    match self.code {
      Catcode::SmuggleTHE => {
        // LaTeXML Bug, we haven't correctly emulated scan_toks! Offending token was:
        let msg = s!(
          "We are marking as \\noexpand a masked \\the-produced token, this must Never happen. \
           Illegal: {}",
          &self.stringify()
        );
        Fatal!(Parameter, Unexpected, None, state, msg);
      },
      Catcode::CS | Catcode::ACTIVE => {
        if state.is_dont_expandable(&self) {
          Ok(Token {
            text: arena::pin("\\relax"),
            code: Catcode::CS,
            smuggled: Some(Box::new(self)),
          })
        } else {
          Ok(self)
        }
      },
      _ => Ok(self),
    }
  }

  /// Return the original token of a not-expanded token,
  /// or undef if it isn't marked as such.
  pub fn get_dont_expand(&self) -> &Option<Box<Token>> { &self.smuggled }
  pub fn get_smuggled(&self) -> &Option<Box<Token>> { &self.smuggled }
  pub fn take_dont_expand(&mut self) -> Option<Box<Token>> { self.smuggled.take() }
  pub fn take_smuggled(&mut self) -> Option<Box<Token>> { self.smuggled.take() }

  /// Remove dont_expand flag, remove SMUGGLE_THE wrapper
  pub fn without_dont_expand(mut self) -> Token {
    match self.smuggled.take() {
      Some(t) => *t,
      None => self,
    }
  }
  /// Borrow a smuggled inner token, without consuming the owner Token
  pub fn without_dont_expand_ref(&self) -> &Token {
    match &self.smuggled {
      Some(t) => t,
      None => self,
    }
  }
  ///======================================================================
  /// Note that this converts the string to a more `user readable' form using `standard' chars for
  /// catcodes. We'll need to be careful about using string instead of reverting for internal
  /// purposes where the
  /// actual character is needed.

  // A Token reverts to itself
  pub fn revert(self) -> Token { self }

  /// A string form which is primarily used for error-reporting
  pub fn stringify(&self) -> String {
    let mut string = self.with_str(|text| text.to_string());
    // Make the token's char content more printable, since this is for a visual messages.
    if string.len() == 1 {
      let c = string.chars().next().unwrap() as u16;
      if c < 0x020 {
        // TODO: sprintf("%04x", c)
        string = s!("U+{}/{}", c, CONTROLNAME[c as usize]);
      }
    }
    let smuggled = self.smuggled.as_ref().map(|t| s!("<{}>",t.stringify())).unwrap_or_default();
    s!("{}[{}]{}", self.code.short_name(), string, smuggled)
  }

  pub fn to_register(&self, state: &State) -> Option<Arc<Register>> {
    state.lookup_register_definition(self)
  }

  pub fn to_number(&self) -> Number { Number::new(self.with_str(|text| text.parse::<i64>()).unwrap_or(0)) }

  pub fn to_dimension(&self) -> Dimension {
    Dimension::new_f64(self.with_str(|text| text.parse::<f64>().unwrap_or(0.0)))
  }

  pub fn to_mu_dimension(&self) -> MuDimension {
    MuDimension::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn to_glue(&self) -> Glue { Glue::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0)) }

  pub fn to_mu_glue(&self) -> MuGlue {
    MuGlue::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn to_float(&self) -> Float {
    Float::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn be_digested(self, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
    stomach.digest(Tokens::new(vec![self]), state)
  }
}
