use lazy_static::lazy_static;
use log::warn;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Display;
use std::iter::FromIterator;
use std::rc::Rc;

use crate::common::dimension::{Dimension, MuDimension};
use crate::common::error::*;
use crate::common::glue::{Glue, MuGlue};
use crate::common::number::Number;
use crate::common::store::Stored;
use crate::definition::register::NumericOps;
use crate::definition::register::{RegisterValue, RegisterCell};
use crate::definition::Definition;
use crate::state::State;
use crate::stomach::Stomach;
use crate::tokens::Tokens;
use crate::{BoxOps, Digested};

lazy_static! {
  pub static ref MOCK_TOKEN: Token = Token::default();
}

static CONTROLNAME : &[&str] = &["NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL", "BS", "HT", "LF", "VT", "FF", "CR", "SO", "SI", "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN", "EM", "SUB", "ESC", "FS", "GS", "RS", "US"];


#[derive(PartialEq, Clone, Copy, Hash, Debug)]
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
  NOTEXPANDED,
  MARKER,
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
      17 => NOTEXPANDED,
      18 => MARKER,
      _ => {
        warn!(target:"unknown:catcode", "Unrecognized catcode: {:?}", num);
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
      NOTEXPANDED => 17,
      MARKER => 18,
    }
  }
}

impl Catcode {
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
      NOTEXPANDED => "NotExpanded",
      // Non-primitive
      IGNORE => "Ignore",
      LETTER => "Letter",
      OTHER => "Other",
      ACTIVE => "Active",
      COMMENT => "Comment",
      INVALID => "Invalid",
      CS => "ControlSequence",
      MARKER => "Marker",
    }
  }

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

  pub fn short_name(&self) -> &'static str {
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
      NOTEXPANDED => "T_NOTEXPANDED",
      MARKER => "T_MARKER"
    }
  }

  // ======================================================================
  // Categories of Category codes.
  // For Tokens with these catcodes, only the catcode is relevant for comparison.
  // (if they even make it to a stage where they get compared)
  pub fn is_primitive(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Primitives
      ESCAPE | BEGIN | END | MATH | ALIGN | EOL | PARAM | SUPER | SUB | SPACE | NOTEXPANDED => true,
      // Non-primitive
      IGNORE | LETTER | OTHER | ACTIVE | COMMENT | INVALID | CS | MARKER => false,
    }
  }

  pub fn is_executable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Executable
      BEGIN | END | MATH | ALIGN | SUPER | SUB | ACTIVE | CS => true,
      // Non-executable
      EOL | ESCAPE | PARAM | SPACE | IGNORE | LETTER | OTHER | COMMENT | INVALID | NOTEXPANDED | MARKER => false,
    }
  }

  pub fn is_neutralizable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Neutralizable
      MATH | ALIGN | PARAM | SUPER | SUB | ACTIVE => true,
      // Non-neutralizable
      ESCAPE | BEGIN | END | EOL | IGNORE | SPACE | LETTER | OTHER | COMMENT | INVALID | CS | NOTEXPANDED | MARKER => false,
    }
  }

  pub fn is_active_or_cs(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      ACTIVE | CS => true,
      _ => false,
    }
  }

  pub fn is_absorbable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Absorbable
      SPACE | LETTER | OTHER | COMMENT => true,
      _ => false,
    }
  }
}

/// The core immutable syntactic primitive resulting from TeX's read-in and expansion process
/// We allow the fields to be public, so that we can use builder macros such as
/// ```
/// macro_rules! T_SPACE(() => {
///     Token { text: Cow::Borrowed(" "),code: Catcode::SPACE}
///   });
/// ```
#[derive(Clone)]
pub struct Token {
  pub text: Cow<'static, str>,
  pub code: Catcode,
}

impl fmt::Debug for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self.text) }
}

impl Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.text) }
}

/// Compare two tokens; They are equal if they both have same catcode & string
// [We pretend all SPACE's are the same, since we'd like to hide newline's in there!]
// NOTE: That another popular equality checks whether the "meaning" (defn) are the same.
// That is NOT done here; see Equals(x,y) and XEquals(x,y)
impl PartialEq for Token {
  fn eq(&self, other: &Token) -> bool { self.code == other.code && (self.code == Catcode::SPACE || (*self.text == *other.text)) }
}

#[macro_export]
macro_rules! T_BEGIN(() => {
  Token { text: Cow::Borrowed("{"),code: Catcode::BEGIN}
});

#[macro_export]
macro_rules! T_END(() => {
  Token { text: Cow::Borrowed("}"),code: Catcode::END}
});
#[macro_export]
macro_rules! T_MATH(() => {
  Token { text: Cow::Borrowed("$"),code: Catcode::MATH}
});
#[macro_export]
macro_rules! T_ALIGN(() => {
  Token { text: Cow::Borrowed("&"),code: Catcode::ALIGN}
});
#[macro_export]
macro_rules! T_PARAM(() => {
  Token { text: Cow::Borrowed("#"),code: Catcode::PARAM}
});
#[macro_export]
macro_rules! T_SUPER(() => {
 Token { text: Cow::Borrowed("^"),code: Catcode::SUPER}
});
#[macro_export]
macro_rules! T_SUB(() => {
  Token { text: Cow::Borrowed("_"),code: Catcode::SUB}
});
#[macro_export]
macro_rules! T_SPACE(() => {
  Token { text: Cow::Borrowed(" "),code: Catcode::SPACE}
};
($text:literal) => {
  Token { text: Cow::Borrowed($text),code: Catcode::SPACE}
});
#[macro_export]
macro_rules! T_CR(() => (
  Token { text: Cow::Borrowed("\n"),code: Catcode::SPACE}
));
#[macro_export]
macro_rules! T_LETTER {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::LETTER,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::LETTER,
    }
  };
}
#[macro_export]
macro_rules! T_OTHER {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::OTHER,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::OTHER,
    }
  };
}
#[macro_export]
macro_rules! T_ACTIVE {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::ACTIVE,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::ACTIVE,
    }
  };
}
#[macro_export]
macro_rules! T_COMMENT {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::COMMENT,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::COMMENT,
    }
  };
}
#[macro_export]
macro_rules! T_CS {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::CS,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::CS,
    }
  };
}
#[macro_export]
macro_rules! T_MARKER {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::MARKER,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::MARKER,
    }
  };
}

#[macro_export]
macro_rules! T_NOTEXPANDED(
  () => {
    Token { text: Cow::Borrowed(""), code: Catcode::NOTEXPANDED }
  };
  ($text:literal) => { Token { text: Cow::Borrowed($text), code: Catcode::NOTEXPANDED } };
  ($text:expr) => { Token { text: Cow::Owned($text.to_string()), code: Catcode::NOTEXPANDED } }
);

#[macro_export]
macro_rules! Token {
  ($text:literal, $cc_opt:expr) => {
    Token {
      text: Cow::Borrowed($text),
      code: match $cc_opt {
        Some(cc) => cc,
        None => Catcode::OTHER,
      },
    }
  };
  ($text:expr, $cc_opt:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: match $cc_opt {
        Some(cc) => cc,
        None => Catcode::OTHER,
      },
    }
  };
}

impl Default for Token {
  fn default() -> Self { T_OTHER!("") }
}

// Explode a string into a list of tokens, all w/catcode OTHER (except space).
#[macro_export]
macro_rules! Explode(($text:expr) => (
  $text.to_string().as_str().chars().map(|c|
    if c==' ' { T_SPACE!() }
    else { T_OTHER!(c) }
  ).collect::<Vec<Token>>()
));

// Similar to Explode, but convert letters to catcode LETTER and others to OTHER
// Hopefully, this is essentially correct WITHOUT resorting to catcode lookup?
#[macro_export]
macro_rules! ExplodeText(($text:expr) => (
  $text.to_string().as_str().chars().map(|c|
    if c==' ' { T_SPACE!() }
    else if c.is_alphabetic() { T_LETTER!(c) }
    else { T_OTHER!(c) }
  ).collect::<Vec<Token>>()
));

static UNTEX_LINELENGTH: usize = 78; // [CONSTANT]
pub fn untex(digested: &Digested, state: &State) -> Result<String> {
  use crate::token::Catcode::*;
  let mut tokens = VecDeque::from_iter(digested.revert()?.unlist().into_iter());
  let mut tex_string = String::new();
  let mut length = 0;
  let mut level: i32 = 0;
  let mut prevs = String::new();
  let mut prevcc = COMMENT;
  while let Some(token) = tokens.pop_front() {
    let cc = token.get_catcode();
    if cc == COMMENT {
      continue;
    }
    let mut token_string = token.get_string().to_owned();
    let first_char = match token_string.chars().next() {
      Some(c) => c,
      None => '\n', // Note: only-used to fail alphanumeric test
    };
    if cc == LETTER {
      // keep "words" together, just for aesthetics
      while !tokens.is_empty() && tokens[0].get_catcode() == LETTER {
        token_string += tokens.pop_front().unwrap().get_string()
      }
    }
    let l = token_string.len();
    if cc == BEGIN {
      level += 1;
    }
    // Seems a reasonable & safe time to line break, for readability, etc.
    if (cc == SPACE) && (token_string == "\n") {
      // preserve newlines already present
      if length > 0 {
        tex_string.push_str(&token_string);
        length = 0;
      }
    }
    // If this token is a letter (or otherwise starts with a letter or digit): space or linebreak
    else if ((cc == LETTER) || ((cc == OTHER) && first_char.is_alphanumeric()))
      && (prevcc == CS)
      && (!prevs.is_empty())
      && (state.lookup_catcode(prevs.chars().rev().next().unwrap()) == Some(LETTER))
    {
      // Insert a (virtual) space before a letter if previous token was a CS w/letters
      // This is required for letters, but just aesthetic for digits (to me?)
      // Of course, use a newline if we're already at end
      let space = if (length > 0) && (length + l > UNTEX_LINELENGTH) { '\n' } else { ' ' };
      tex_string.push(space);
      tex_string.push_str(&token_string);
      length += 1 + l;
    } else if (length > 0) && (length + l > UNTEX_LINELENGTH) && (tokens.len() > 1) {
      // linebreak before this token?
      // and not at end! Or even within an arg!
      tex_string.push_str("%\n");
      tex_string.push_str(&token_string);
      length = l; // with %, so that it "disappears"
    } else {
      tex_string.push_str(&token_string);
      length += l;
    }
    if cc == END {
      level -= 1;
    }

    prevs = token_string;
    prevcc = cc;
  }
  // Patch up nesting for valid TeX !!!
  if level > 0 {
    let close_brace_string = String::from_utf8(vec![b'}'; level.abs() as usize]).unwrap();
    tex_string = tex_string + &close_brace_string;
  } else if level < 0 {
    let open_brace_string = String::from_utf8(vec![b'{'; level.abs() as usize]).unwrap();
    tex_string = open_brace_string + &tex_string;
  }
  Ok(tex_string)
}

// TODO: Skipped ...

///======================================================================
/// Accessors.
impl<'a> Token {
  pub fn new(text: Cow<'static, str>, code: Catcode) -> Self { Token { text, code } }
  pub fn isa_token(&self) -> bool { true }

  /// Get the CS Name of the token. This is the name that definitions will be
  /// stored under; It's the same for various `different' BEGIN tokens, eg.
  pub fn get_cs_name(&'a self) -> &'a str {
    if self.code.is_primitive() {
      self.code.name()
    } else {
      &self.text
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
      self.get_primitive_name().unwrap_or_else(|| &self.text).to_string()
    } else {
      String::new()
    }
  }

  /// Return the string or character part of the token
  pub fn get_string(&self) -> &str { &self.text }

  /// Return the character code of  character part of the token, or 256 if it is a control
  /// sequence
  pub fn get_charcode(&self) -> u32 {
    if self.code == Catcode::CS {
      256
    } else if let Some(c) = self.text.chars().next() {
      c as u32
    } else {
      0
    }
  }

  /// Return the catcode of the token.
  pub fn get_catcode(&self) -> Catcode { self.code }

  pub fn is_executable(&self) -> bool { self.code.is_executable() }

  /// Defined so a Token or Tokens can be used interchangeably.
  pub fn unlist(&self) -> Vec<Token> { vec![self.clone()] }

  /// neutralize really should only retroactively imitate what Semiverbatim would have done.
  /// So, it needs to neutralize those in SPECIALS
  /// NOTE that although '%' gets it's catcode changed in Semiverbatim,
  /// I'm pretty sure we do NOT want to neutralize comments (turn them into CC_OTHER)
  /// here, since if comments do get into the Tokens, that will introduce weird crap into the
  /// stream.
  pub fn neutralize(self, extraspecials: &[Token], state: &State) -> Token {
    let ch = match self.text.chars().next() {
      Some(ch) => ch,
      None => return self,
    };
    let cc = self.code;
    if cc.is_neutralizable() {
      let mut is_special = false;
      if let Some(specials_store) = state.lookup_value("SPECIALS") {
        let evec = Vec::new();
        let specials_list: &Vec<char> = match *specials_store {
          Stored::VecChar(ref list) => list,
          _ => &evec,
        };
        for special in specials_list.iter() {
          if *special == ch {
            is_special = true;
            break;
          }
        }
      }
      if is_special || !extraspecials.is_empty() {
        T_OTHER!(self.text)
      } else {
        self
      }
    } else {
      self
    }
  }

  ///======================================================================
  /// Note that this converts the string to a more `user readable' form using `standard' chars for
  /// catcodes. We'll need to be careful about using string instead of reverting for internal
  /// purposes where the
  /// actual character is needed.

  /// Should revert do something with this???
  ///  ($standardchar[$$self[1]] || $$self[0]); }

  pub fn revert(&self) -> Token { self.clone() }

  pub fn as_str(&self) -> &str { &self.text }

  pub fn stringify(&self) -> String {
    let mut string = self.text.to_string();
    // Make the token's char content more printable, since this is for error messages.
    if string.len() == 1 {
      let c = string.chars().next().unwrap() as u16;
      if c < 0x020 {
        // TODO: sprintf("%04x", c)
        string = s!("U+{}/{}",c, CONTROLNAME[c as usize]); 
      }
    }
    s!("{}[{}]", self.code.short_name(), string)
  }

  pub fn to_register(&self, state: &State) -> Option<Rc<RegisterCell>> { state.lookup_register_definition(self) }

  pub fn to_number(&self) -> Number { Number::new(self.text.parse::<f32>().unwrap_or(0.0)) }

  pub fn to_dimension(&self) -> Dimension { Dimension::new(self.text.parse::<f32>().unwrap_or(0.0)) }

  pub fn to_mu_dimension(&self) -> MuDimension { MuDimension::new(self.text.parse::<f32>().unwrap_or(0.0)) }

  pub fn to_glue(&self) -> Glue { Glue::new(self.text.parse::<f32>().unwrap_or(0.0)) }

  pub fn to_mu_glue(&self) -> MuGlue { MuGlue::new(self.text.parse::<f32>().unwrap_or(0.0)) }

  // TODO: This method may cause more issues than it solves... reconsider?
  // I have already accidentally done token.value_of(Vec::new(), state) and gotten a "0" back
  // when I really meant "token.to_number().value_of()". The token in question was a simple T_OTHER!("3").
  pub fn value_of(&self, args: Vec<Token>, state: &mut State) -> Option<RegisterValue> {
    match self.to_register(state) {
      None => None,
      Some(register) => (*register).value_of(args, state),
    }
  }

  pub fn be_digested(self, stomach: &mut Stomach, state: &mut State) -> Result<Digested> { stomach.digest(Tokens::new(vec![self]), state) }
}
