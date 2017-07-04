use std::iter::FromIterator;
use std::collections::VecDeque;

use common::error::*;
use state::{State, ObjectStore};
use {Digested, BoxOps};
use stomach::Stomach;
use quote;
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
impl quote::ToTokens for Catcode {
  fn to_tokens(&self, tokens: &mut quote::Tokens) {
    use token::Catcode::*;
    let verbatim = match *self {
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
    tokens.append("Catcode");
    tokens.append("::");
    tokens.append(verbatim);
  }
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
      ESCAPE |
      BEGIN |
      END |
      MATH |
      ALIGN |
      EOL |
      PARAM |
      SUPER |
      SUB |
      SPACE |
      NOTEXPANDED => true,
      // Non-primitive
      IGNORE |
      LETTER |
      OTHER |
      ACTIVE |
      COMMENT |
      INVALID |
      CS |
      MARKER => false,
    }
  }

  pub fn is_executable(&self) -> bool {
    use token::Catcode::*;
    match *self {
      // Executable
      BEGIN |
      END |
      MATH |
      ALIGN |
      SUPER |
      SUB |
      ACTIVE |
      CS => true,
      // Non-executable
      EOL |
      ESCAPE |
      PARAM |
      SPACE |
      IGNORE |
      LETTER |
      OTHER |
      COMMENT |
      INVALID |
      NOTEXPANDED |
      MARKER => false,
    }
  }

  pub fn is_neutralizable(&self) -> bool {
    use token::Catcode::*;
    match *self {
      // Neutralizable
      MATH |
      ALIGN |
      PARAM |
      SUPER |
      SUB |
      ACTIVE => true,
      // Non-neutralizable
      ESCAPE |
      BEGIN |
      END |
      EOL |
      IGNORE |
      SPACE |
      LETTER |
      OTHER |
      COMMENT |
      INVALID |
      CS |
      NOTEXPANDED |
      MARKER => false,
    }
  }

  pub fn is_active_or_cs(&self) -> bool {
    use token::Catcode::*;
    match *self {
      ACTIVE | CS => true,
      _ => false
    }
  }

}
#[derive(Clone, Hash, Debug, PartialEq)]
pub struct Token {
  pub text: String,
  pub code: Catcode,
}
impl quote::ToTokens for Token {
  fn to_tokens(&self, tokens: &mut quote::Tokens) {
    tokens.append("Token");
    tokens.append("{");
    tokens.append("text:");
    self.text.to_tokens(tokens);
    tokens.append(".to_string(), code: ");
    self.code.to_tokens(tokens);
    tokens.append("}")
  }
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

static UNTEX_LINELENGTH : usize = 78; // [CONSTANT]
pub fn untex(digested: &Digested, state: &State) -> String {
  use token::Catcode::*;
  let mut tokens = VecDeque::from_iter(digested.revert().into_iter());
  let mut tex_string = String::new();
  let mut length = 0;
  let mut level : i32 = 0;
  let mut prevs = String::new();
  let mut prevcc = COMMENT;
  while let Some(token) = tokens.pop_front() {
    let cc = token.get_catcode();
    if cc == COMMENT { continue; }
    let mut token_string = token.get_string().to_owned();
    let first_char = match token_string.chars().next() {
      Some(c) => c,
      None => '\n' // Note: only-used to fail alphanumeric test
    };
    if cc == LETTER {    // keep "words" together, just for aesthetics
      while !tokens.is_empty() && tokens.get(0).unwrap().get_catcode() == LETTER {
        token_string += tokens.pop_front().unwrap().get_string()
      }
    }
    let l = token_string.len();
    if cc == BEGIN { level += 1; }
    // Seems a reasonable & safe time to line break, for readability, etc.
    if (cc == SPACE) && (token_string == "\n") {    // preserve newlines already present
      if length > 0 {
        tex_string.push_str(&token_string);
        length = 0;
      }
    }
    // If this token is a letter (or otherwise starts with a letter or digit): space or linebreak
    else if ((cc == LETTER) || ((cc == OTHER) && first_char.is_alphanumeric()))
      && (prevcc == CS) && (!prevs.is_empty())
      && (state.lookup_catcode(&prevs.chars().rev().next().unwrap()) == Some(LETTER)) {
      // Insert a (virtual) space before a letter if previous token was a CS w/letters
      // This is required for letters, but just aesthetic for digits (to me?)
      // Of course, use a newline if we're already at end
      let space = if (length > 0) && (length + l > UNTEX_LINELENGTH) {
        '\n'
      } else {
        ' '
      };
      tex_string.push(space);
      tex_string.push_str(&token_string);
      length += 1 + l;
    } else if (length > 0) && (length + l > UNTEX_LINELENGTH)    // linebreak before this token?
      && (tokens.len() > 1) {                                 // and not at end! Or even within an arg!
      tex_string.push_str("%\n");
      tex_string.push_str(&token_string);
      length = l;                                                // with %, so that it "disappears"
    } else {
      tex_string.push_str(&token_string);
      length += l;
    }
    if cc == END { level -= 1; }

    prevs = token_string;
    prevcc = cc;
  }
  // Patch up nesting for valid TeX !!!
  if level > 0 {
    let close_brace_string = String::from_utf8(vec![b'}'; level.abs() as usize]).unwrap();
    tex_string = tex_string + &close_brace_string;
  }
  else if level < 0 {
    let open_brace_string = String::from_utf8(vec![b'{'; level.abs() as usize]).unwrap();
    tex_string = open_brace_string + &tex_string;
  }

  tex_string
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
  pub fn neutralize(self, extraspecials : &[Token], state: &State) -> Token {
    let ch = match self.text.chars().next() {
      Some(ch) => ch,
      None => return self
    };
    let cc = self.code;
    if cc.is_neutralizable() {
      let mut is_special = false;
      if let Some(specials_store) = state.lookup_value("SPECIALS") {
        let evec = Vec::new();
        let specials_list : &Vec<char> = match *specials_store {
          ObjectStore::VecChar(ref list) => list,
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
