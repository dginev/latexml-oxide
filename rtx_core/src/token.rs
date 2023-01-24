use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Display;
use std::sync::Arc;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::glue::Glue;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::register::RegisterCell;
use crate::state::State;
use crate::stomach::Stomach;
use crate::tokens::Tokens;
use crate::{fatal, Digested, Fatal};

static CONTROLNAME: &[&str] = &[
  "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL", "BS", "HT", "LF", "VT", "FF", "CR", "SO", "SI", "DLE", "DC1", "DC2", "DC3", "DC4", "NAK",
  "SYN", "ETB", "CAN", "EM", "SUB", "ESC", "FS", "GS", "RS", "US",
];

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
  pub fn is_primitive(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Primitives
      ESCAPE | BEGIN | END | MATH | ALIGN | EOL | PARAM | SUPER | SUB | SPACE => true,
      // Non-primitive
      IGNORE | LETTER | OTHER | ACTIVE | COMMENT | INVALID | CS | MARKER | ARG | SmuggleTHE => false,
    }
  }

  pub fn is_executable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Executable
      BEGIN | END | MATH | ALIGN | SUPER | SUB | ACTIVE | CS => true,
      // Non-executable
      EOL | ESCAPE | PARAM | SPACE | IGNORE | LETTER | OTHER | COMMENT | INVALID | MARKER | ARG | SmuggleTHE => false,
    }
  }

  pub fn is_neutralizable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Neutralizable
      MATH | ALIGN | PARAM | SUPER | SUB | ACTIVE => true,
      // Non-neutralizable
      ESCAPE | BEGIN | END | EOL | IGNORE | SPACE | LETTER | OTHER | COMMENT | INVALID | CS | MARKER | ARG | SmuggleTHE => false,
    }
  }

  pub fn is_active_or_cs(self) -> bool {
    use crate::token::Catcode::*;
    matches!(self, ACTIVE | CS)
  }

  pub fn is_absorbable(self) -> bool {
    use crate::token::Catcode::*;
    // Absorbable
    matches!(self, SPACE | LETTER | OTHER | COMMENT)
  }

  pub fn is_gullet_holdable(self) -> bool {
    use crate::token::Catcode::*;
    matches!(self, COMMENT | MARKER)
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
  pub smuggled: Option<Box<Token>>,
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
  fn eq(&self, other: &Token) -> bool {
    self.code == other.code && (self.code == Catcode::SPACE || (*self.text == *other.text)) && self.smuggled.is_none() == other.smuggled.is_none()
  }
}

#[macro_export]
macro_rules! T_BEGIN(() => {
  Token { text: Cow::Borrowed("{"),code: Catcode::BEGIN, smuggled: None}
});

#[macro_export]
macro_rules! T_END(() => {
  Token { text: Cow::Borrowed("}"),code: Catcode::END, smuggled: None}
});
#[macro_export]
macro_rules! T_MATH(() => {
  Token { text: Cow::Borrowed("$"),code: Catcode::MATH, smuggled: None}
});
#[macro_export]
macro_rules! T_ALIGN(() => {
  Token { text: Cow::Borrowed("&"),code: Catcode::ALIGN, smuggled: None}
});
#[macro_export]
macro_rules! T_PARAM(() => {
  Token { text: Cow::Borrowed("#"),code: Catcode::PARAM, smuggled: None}
});
#[macro_export]
macro_rules! T_SUPER(() => {
 Token { text: Cow::Borrowed("^"),code: Catcode::SUPER, smuggled: None}
});
#[macro_export]
macro_rules! T_SUB(() => {
  Token { text: Cow::Borrowed("_"),code: Catcode::SUB, smuggled: None}
});
#[macro_export]
macro_rules! T_SPACE(() => {
  Token { text: Cow::Borrowed(" "),code: Catcode::SPACE, smuggled: None}
};
($text:literal) => {
  Token { text: Cow::Borrowed($text),code: Catcode::SPACE, smuggled: None}
});
#[macro_export]
macro_rules! T_CR(() => (
  Token { text: Cow::Borrowed("\n"),code: Catcode::SPACE, smuggled: None}
));
#[macro_export]
macro_rules! T_LETTER {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::LETTER,
      smuggled: None,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::LETTER,
      smuggled: None,
    }
  };
}
#[macro_export]
macro_rules! T_OTHER {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::OTHER,
      smuggled: None,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::OTHER,
      smuggled: None,
    }
  };
}
#[macro_export]
macro_rules! T_ACTIVE {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::ACTIVE,
      smuggled: None,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::ACTIVE,
      smuggled: None,
    }
  };
}
#[macro_export]
macro_rules! T_COMMENT {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::COMMENT,
      smuggled: None,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::COMMENT,
      smuggled: None,
    }
  };
}
#[macro_export]
macro_rules! T_CS {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::CS,
      smuggled: None,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::CS,
      smuggled: None,
    }
  };
}
#[macro_export]
macro_rules! T_MARKER {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::MARKER,
      smuggled: None,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::MARKER,
      smuggled: None,
    }
  };
}

#[macro_export]
macro_rules! T_ARG {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::ARG,
      smuggled: None,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::ARG,
      smuggled: None,
    }
  };
}

#[macro_export]
macro_rules! Token {
  ($text:literal) => {
    Token {
      text: Cow::Borrowed($text),
      code: Catcode::OTHER,
      smuggled: None,
    }
  };
  ($text:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: Catcode::OTHER,
      smuggled: None,
    }
  };
  ($text:literal, $cc:expr) => {
    Token {
      text: Cow::Borrowed($text),
      code: $cc,
      smuggled: None,
    }
  };
  ($text:expr, $cc:expr) => {
    Token {
      text: Cow::Owned($text.to_string()),
      code: $cc,
      smuggled: None,
    }
  };
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
pub fn untex_digested(digested: &Digested, suppress_linebreak: bool, state: &mut State) -> Result<String> {
  untex(digested.revert(state)?, suppress_linebreak, state)
}
pub fn untex(tokens: Tokens, _suppress_linebreak: bool, state: &mut State) -> Result<String> {
  use crate::token::Catcode::*;
  let mut tokens: VecDeque<Token> = tokens.unlist().into_iter().collect();
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
    // Note: \n only-used to fail alphanumeric test
    let first_char = token_string.chars().next().unwrap_or('\n');
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
  match level {
    0 => {},
    1i32..=std::i32::MAX => {
      let close_brace_string = String::from_utf8(vec![b'}'; level.unsigned_abs() as usize]).unwrap();
      tex_string = tex_string + &close_brace_string;
    },
    std::i32::MIN..=-1i32 => {
      let open_brace_string = String::from_utf8(vec![b'{'; level.unsigned_abs() as usize]).unwrap();
      tex_string = open_brace_string + &tex_string;
    },
  };
  Ok(tex_string)
}

// TODO: Skipped ...

///======================================================================
/// Accessors.
impl<'a> Token {
  pub fn new(text: Cow<'static, str>, code: Catcode) -> Self { Token { text, code, smuggled: None } }
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
      self.get_primitive_name().unwrap_or(&self.text).to_string()
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

  /// Return the the borrowed &str "text" of the token
  /// use `to_string` instead for an owned String
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
  pub fn neutralize(self, extraspecials: &[char], state: &State) -> Token {
    let ch = match self.text.chars().next() {
      Some(ch) => ch,
      None => return self,
    };
    let cc = self.code;
    if cc.is_neutralizable() {
      for extra in extraspecials {
        if extra == &ch {
          return T_OTHER!(ch);
        }
      }
      if let Some(Stored::VecChar(ref specials_list)) = state.lookup_value("SPECIALS") {
        for special in specials_list.iter() {
          if *special == ch {
            return T_OTHER!(ch);
          }
        }
      }
    }
    self
  }

  pub fn substitute_parameters(self, args: &[&Token]) -> Self {
    if self.code == Catcode::ARG {
      args[self.text.parse::<usize>().unwrap() - 1].clone()
    } else {
      self
    }
  }

  pub fn pack_parameters(self) -> Self { self }

  pub fn with_dont_expand(self, state: &State) -> Result<Self> {
    let cc = self.code;
    if cc == Catcode::SmuggleTHE {
      // LaTeXML Bug, we haven't correctly emulated scan_toks! Offending token was:
      let msg = s!(
        "We are marking as \\noexpand a masked \\the-produced token, this must Never happen. Illegal: {}",
        &self.stringify()
      );
      Fatal!(Parameter, Unexpected, None, state, msg);
    }
    if (cc == Catcode::CS || cc == Catcode::ACTIVE) && state.is_dont_expandable(&self) {
      Ok(Token {
        text: Cow::Borrowed("\\relax"),
        code: Catcode::CS,
        smuggled: Some(Box::new(self)),
      })
    } else {
      Ok(self)
    }
  }

  /// Return the original token of a not-expanded token,
  /// or undef if it isn't marked as such.
  pub fn get_dont_expand(&self) -> &Option<Box<Token>> { &self.smuggled }

  /// Remove dont_expand flag, remove SMUGGLE_THE wrapper
  pub fn without_dont_expand(mut self) -> Token {
    let mut inner = self.smuggled.take();
    match inner {
      Some(t) => *t,
      None => self,
    }
  }

  ///======================================================================
  /// Note that this converts the string to a more `user readable' form using `standard' chars for
  /// catcodes. We'll need to be careful about using string instead of reverting for internal
  /// purposes where the
  /// actual character is needed.

  /// Should revert do something with this???
  ///  ($standardchar[$$self[1]] || $$self[0]); }

  pub fn revert(self) -> Token { self }

  pub fn as_str(&self) -> &str { &self.text }

  pub fn stringify(&self) -> String {
    let mut string = self.text.to_string();
    // Make the token's char content more printable, since this is for error messages.
    if string.len() == 1 {
      let c = string.chars().next().unwrap() as u16;
      if c < 0x020 {
        // TODO: sprintf("%04x", c)
        string = s!("U+{}/{}", c, CONTROLNAME[c as usize]);
      }
    }
    s!("{}[{}]", self.code.short_name(), string)
  }

  pub fn to_register(&self, state: &State) -> Option<Arc<RegisterCell>> { state.lookup_register_definition(self) }

  pub fn to_number(&self) -> Number { Number::new(self.text.parse::<i32>().unwrap_or(0)) }

  pub fn to_dimension(&self) -> Dimension { Dimension::new_f32(self.text.parse::<f32>().unwrap_or(0.0)) }

  pub fn to_mu_dimension(&self) -> MuDimension { MuDimension::new_f32(self.text.parse::<f32>().unwrap_or(0.0)) }

  pub fn to_glue(&self) -> Glue { Glue::new_f32(self.text.parse::<f32>().unwrap_or(0.0)) }

  pub fn to_mu_glue(&self) -> MuGlue { MuGlue::new_f32(self.text.parse::<f32>().unwrap_or(0.0)) }

  pub fn be_digested(self, stomach: &mut Stomach, state: &mut State) -> Result<Digested> { stomach.digest(Tokens::new(vec![self]), state) }
}
