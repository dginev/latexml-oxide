use core::Digested;
use std::fmt;

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

#[derive(Clone, Hash, Debug)]
pub struct Token{
 pub text : String,
 pub code : Catcode
}

pub fn T_BEGIN() -> Token {
  Token { text: "{".to_string(), code: Catcode::BEGIN }  
}
pub fn T_END() -> Token {
  Token { text: "}".to_string(), code: Catcode::END }
}
pub fn T_MATH() -> Token {
  Token { text: "$".to_string(), code: Catcode::MATH }
}
pub fn T_ALIGN() -> Token {
  Token { text: "&".to_string(), code: Catcode::ALIGN }
}
pub fn T_PARAM() -> Token {
  Token { text: "#".to_string(), code: Catcode::PARAM }  
}
pub fn T_SUPER() -> Token {
 Token { text: "^".to_string(), code: Catcode::SUPER }
}
pub fn T_SUB() -> Token {
  Token { text: "_".to_string(), code: Catcode::SUB }
}
pub fn T_SPACE() -> Token {
  Token { text: " ".to_string(), code: Catcode::SPACE }
}
pub fn T_CR() -> Token {
  Token { text: "\n".to_string(), code: Catcode::SPACE }
}
pub fn T_LETTER(text : String) -> Token {
  Token { text : text, code: Catcode::LETTER }
}
pub fn T_OTHER(text : String) -> Token {
  Token { text : text, code: Catcode::OTHER }
}
pub fn T_ACTIVE(text : String) -> Token {
  Token { text : text, code: Catcode::ACTIVE }
}
pub fn T_COMMENT(text : String) -> Token {
  Token { text : "%".to_string(), code: Catcode::ACTIVE }
}
pub fn T_CS(text : String) -> Token {
  Token { text : text.to_string(), code: Catcode::CS}
}
pub fn T_MARKER(text : String) -> Token {
  Token { text : text.to_string(), code: Catcode::MARKER}
}

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
    if c==' ' { T_SPACE() }
    else { T_OTHER(c.to_string()) }
  ).collect()
}

// Similar to Explode, but convert letters to catcode LETTER and others to OTHER
// Hopefully, this is essentially correct WITHOUT resorting to catcode lookup?
pub fn ExplodeText(text : String) -> Vec<Token> {
  text.chars().map(|c| 
    if c==' ' { T_SPACE() }
    else if c.is_alphabetic() { T_LETTER(c.to_string()) }
    else { T_OTHER(c.to_string()) }
  ).collect::<Vec<Token>>()
}

pub fn untex(digested : Digested) -> String {
  digested.to_string()
}