use common::error::*;
use state::{State, ObjectStore};
use {Digested, BoxOps};
use stomach::Stomach;
use tokens::Tokens;

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
impl Catcode {
  pub fn name(&self) -> String {
    use token::Catcode::*;
    match *self {
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
    .to_string()
  }

  // ======================================================================
  // Categories of Category codes.
  // For Tokens with these catcodes, only the catcode is relevant for comparison.
  // (if they even make it to a stage where they get compared)
  pub fn is_primitive(&self) -> bool {
    use token::Catcode::*;
    match *self {
      // Primitives
      ESCAPE => true,
      BEGIN => true,
      END => true,
      MATH => true,
      ALIGN => true,
      EOL => true,
      PARAM => true,
      SUPER => true,
      SUB => true,
      SPACE => true,
      NOTEXPANDED => true,
      // Non-primitive
      IGNORE => false,
      LETTER => false,
      OTHER => false,
      ACTIVE => false,
      COMMENT => false,
      INVALID => false,
      CS => false,
      MARKER => false,
    }
  }

  pub fn is_executable(&self) -> bool {
    use token::Catcode::*;
    match *self {
      // Executable
      BEGIN => true,
      END => true,
      MATH => true,
      ALIGN => true,
      SUPER => true,
      SUB => true,
      ACTIVE => true,
      CS => true,
      // Non-executable
      EOL => false,
      ESCAPE => false,
      PARAM => false,
      SPACE => false,
      IGNORE => false,
      LETTER => false,
      OTHER => false,
      COMMENT => false,
      INVALID => false,
      NOTEXPANDED => false,
      MARKER => false,
    }
  }

  pub fn is_neutralizable(&self) -> bool {
    use token::Catcode::*;
    match *self {
      // Neutralizable
      MATH => true,
      ALIGN => true,
      PARAM => true,
      SUPER => true,
      SUB => true,
      ACTIVE => true,
      // Non-neutralizable
      ESCAPE => false,
      BEGIN => false,
      END => false,
      EOL => false,
      IGNORE => false,
      SPACE => false,
      LETTER => false,
      OTHER => false,
      COMMENT => false,
      INVALID => false,
      CS => false,
      NOTEXPANDED => false,
      MARKER => false,
    }
  }

  pub fn is_active_or_cs(&self) -> bool {
    use token::Catcode::*;
    match *self {
      ACTIVE => true,
      CS => true,
      _ => false
    }
  }

}
#[derive(Clone, Hash, Debug, PartialEq)]
pub struct Token {
  pub text: String,
  pub code: Catcode,
}

#[macro_export]
macro_rules! T_BEGIN(() => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text: "{".to_string(), code: Catcode::BEGIN }
}));

#[macro_export]
macro_rules! T_END(() => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text: "}".to_string(), code: Catcode::END }
}));
#[macro_export]
macro_rules! T_MATH(() => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text: "$".to_string(), code: Catcode::MATH }
}));
#[macro_export]
macro_rules! T_ALIGN(() => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text: "&".to_string(), code: Catcode::ALIGN }
}));
#[macro_export]
macro_rules! T_PARAM(() => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text: "#".to_string(), code: Catcode::PARAM }
}));
#[macro_export]
macro_rules! T_SUPER(() => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
 Token { text: "^".to_string(), code: Catcode::SUPER }
}));
#[macro_export]
macro_rules! T_SUB(() => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text: "_".to_string(), code: Catcode::SUB }
}));
#[macro_export]
macro_rules! T_SPACE(() => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text: " ".to_string(), code: Catcode::SPACE }
}));
#[macro_export]
macro_rules! T_CR(() => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text: "\n".to_string(), code: Catcode::SPACE }
}));
#[macro_export]
macro_rules! T_LETTER(($text:expr) => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text : $text.to_string(), code: Catcode::LETTER }
}));
#[macro_export]
macro_rules! T_OTHER(($text:expr) => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text : $text.to_string(), code: Catcode::OTHER }
}));
#[macro_export]
macro_rules! T_ACTIVE(($text:expr) => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text : $text.to_string(), code: Catcode::ACTIVE }
}));
#[macro_export]
macro_rules! T_COMMENT(($text:expr) => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text : $text.to_string(), code: Catcode::COMMENT }
}));
#[macro_export]
macro_rules! T_CS(($text:expr) => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text : $text.to_string(), code: Catcode::CS}
}));
#[macro_export]
macro_rules! T_MARKER(($text:expr) => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text : $text.to_string(), code: Catcode::MARKER}
}));

#[macro_export]
macro_rules! Token(($text:expr, $cc_opt:expr) => ({
  use $crate::token::Token;
  use $crate::token::Catcode;
  Token { text : $text.to_string(),  code: match $cc_opt {
    Some(cc) => cc,
    None => Catcode::OTHER
  }}
}));

// Explode a string into a list of tokens, all w/catcode OTHER (except space).
#[macro_export]
macro_rules! Explode(($text:expr) => ({
  $text.chars().map(|c|
    if c==' ' { T_SPACE!() }
    else { T_OTHER!(c.to_string()) }
  ).collect()
}));

// Similar to Explode, but convert letters to catcode LETTER and others to OTHER
// Hopefully, this is essentially correct WITHOUT resorting to catcode lookup?
#[macro_export]
macro_rules! ExplodeText(($text:expr) => ({
  $text.chars().map(|c|
    if c==' ' { T_SPACE!() }
    else if c.is_alphabetic() { T_LETTER!(c.to_string()) }
    else { T_OTHER!(c.to_string()) }
  ).collect::<Vec<Token>>()
}));

pub fn untex(digested: Digested) -> String {
  digested.to_string()
}


// TODO: Skipped ...

///======================================================================
/// Accessors.
impl Token {
  pub fn isa_token(&self) -> bool {
    true
  }

  /// Get the CS Name of the token. This is the name that definitions will be
  /// stored under; It's the same for various `different' BEGIN tokens, eg.
  pub fn get_cs_name(&self) -> String {
    if self.code.is_primitive() {
      self.code.name()
    } else {
      self.text.clone()
    }
  }

  /// Get the CS name only if the catcode is executable!
  pub fn get_executable_name(&self) -> String {
    let cc = self.code;
    if cc.is_executable() {
      if cc.is_primitive() {
        cc.name()
      } else {
        self.text.clone()
      }
    } else {
      String::new()
    }
  }

  /// Return the string or character part of the token
  pub fn get_string(&self) -> &str {
    &self.text
  }

  /// Return the character code of  character part of the token, or 256 if it is a control sequence
  pub fn get_charcode(&self) -> u32 {
    if self.code == Catcode::CS {
      256
    }
    else if let Some(c) = self.text.chars().next() {
      c as u32
    } else {
      0
    }
  }

  /// Return the catcode of the token.
  pub fn get_catcode(&self) -> Catcode {
    self.code
  }

  pub fn is_executable(&self) -> bool {
    self.code.is_executable()
  }

  /// Defined so a Token or Tokens can be used interchangeably.
  pub fn unlist(&self) -> Vec<Token> {
    vec![self.clone()]
  }

  /// neutralize really should only retroactively imitate what Semiverbatim would have done.
  /// So, it needs to neutralize those in SPECIALS
  /// NOTE that although '%' gets it's catcode changed in Semiverbatim,
  /// I'm pretty sure we do NOT want to neutralize comments (turn them into CC_OTHER)
  /// here, since if comments do get into the Tokens, that will introduce weird crap into the stream.
  pub fn neutralize(self, extraspecials : &Vec<Token>, state: &State) -> Token {
    let ch = self.text.chars().next().unwrap();
    let cc = self.code;
    if cc.is_neutralizable() {
      let mut is_special = false;
      if let Some(specials_store) = state.lookup_value("SPECIALS") {
        let evec = Vec::new();
        let specials_list : &Vec<char> = match specials_store {
          &ObjectStore::VecChar(ref list) => list,
          _ => &evec
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
  /// Note that this converts the string to a more `user readable' form using `standard' chars for catcodes.
  /// We'll need to be careful about using string instead of reverting for internal purposes where the
  /// actual character is needed.

  /// Should revert do something with this???
  ///  ($standardchar[$$self[1]] || $$self[0]); }

  pub fn revert(&self) -> Token {
    self.clone()
  }

  pub fn to_string(&self) -> String {
    self.text.clone()
  }


  pub fn be_digested(self, stomach : &mut Stomach, state: &mut State) -> Result<Digested> {
    stomach.digest(Tokens{tokens: vec![self]}, state)
  }
}
