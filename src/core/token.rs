use core::Digested;

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
  MARKER
}
impl Catcode {
  pub fn is_primitive(&self) -> bool {
    use core::token::Catcode::*;
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

  pub fn name(&self) -> String {
    use core::token::Catcode::*;
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
      MARKER => "Marker"
    }.to_string()
  }
}

#[derive(Clone, Hash, Debug)]
pub struct Token{
 pub text : String,
 pub code : Catcode
}

#[macro_export]
macro_rules! T_BEGIN(() => ({
  use $crate::core::token::Token;
  Token { text: "{".to_string(), code: Catcode::BEGIN }  
}));

#[macro_export]
macro_rules! T_END(() => ({
  use $crate::core::token::Token;
  Token { text: "}".to_string(), code: Catcode::END }
}));
#[macro_export]
macro_rules! T_MATH(() => ({
  use $crate::core::token::Token;
  Token { text: "$".to_string(), code: Catcode::MATH }
}));
#[macro_export]
macro_rules! T_ALIGN(() => ({
  use $crate::core::token::Token;
  Token { text: "&".to_string(), code: Catcode::ALIGN }
}));
#[macro_export]
macro_rules! T_PARAM(() => ({
  use $crate::core::token::Token;
  Token { text: "#".to_string(), code: Catcode::PARAM }  
}));
#[macro_export]
macro_rules! T_SUPER(() => ({
  use $crate::core::token::Token;
 Token { text: "^".to_string(), code: Catcode::SUPER }
}));
#[macro_export]
macro_rules! T_SUB(() => ({
  use $crate::core::token::Token;
  Token { text: "_".to_string(), code: Catcode::SUB }
}));
#[macro_export]
macro_rules! T_SPACE(() => ({
  use $crate::core::token::Token;
  Token { text: " ".to_string(), code: Catcode::SPACE }
}));
#[macro_export]
macro_rules! T_CR(() => ({
  use $crate::core::token::Token;
  Token { text: "\n".to_string(), code: Catcode::SPACE }
}));
#[macro_export]
macro_rules! T_LETTER(($text:expr) => ({
  Token { text : $text, code: Catcode::LETTER }
}));
#[macro_export]
macro_rules! T_OTHER(($text:expr) => ({
  Token { text : $text, code: Catcode::OTHER }
}));
#[macro_export]
macro_rules! T_ACTIVE(($text:expr) => ({
  Token { text : $text, code: Catcode::ACTIVE }
}));
#[macro_export]
macro_rules! T_COMMENT(($text:expr) => ({
  Token { text : $text, code: Catcode::COMMENT }
}));
#[macro_export]
macro_rules! T_CS(($text:expr) => ({
  Token { text : $text.to_string(), code: Catcode::CS}
}));
#[macro_export]
macro_rules! T_MARKER(($text:expr) => ({
  Token { text : $text.to_string(), code: Catcode::MARKER}
}));

pub fn Token (text : String, cc_opt : Option<Catcode>) -> Token {
  let cc = match cc_opt {
    Some(cc) => cc,
    None => Catcode::OTHER
  };
  Token { text : text,  code: cc }
}
// Explode a string into a list of tokens, all w/catcode OTHER (except space).
pub fn Explode(text : String) -> Vec<Token> {
  text.chars().map(|c| 
    if c==' ' { T_SPACE!() }
    else { T_OTHER!(c.to_string()) }
  ).collect()
}

// Similar to Explode, but convert letters to catcode LETTER and others to OTHER
// Hopefully, this is essentially correct WITHOUT resorting to catcode lookup?
pub fn ExplodeText(text : String) -> Vec<Token> {
  text.chars().map(|c| 
    if c==' ' { T_SPACE!() }
    else if c.is_alphabetic() { T_LETTER!(c.to_string()) }
    else { T_OTHER!(c.to_string()) }
  ).collect::<Vec<Token>>()
}

pub fn untex(digested : Digested) -> String {
  digested.to_string()
}


// TODO: Skipped ...

///======================================================================
/// Accessors.
impl Token {
  pub fn isa_token(&self) -> bool { true }

  /// Get the CS Name of the token. This is the name that definitions will be
  /// stored under; It's the same for various `different' BEGIN tokens, eg.
  pub fn get_cs_name(&self) -> String {
    if self.code.is_primitive() {
      self.code.name()
    } else {
      self.text.clone()
    }
  }

  pub fn to_string(&self) -> String {
    self.text.clone()
  }
}