use core::Digested;

#[derive(PartialEq, Clone)]
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

pub struct Token<'token>{
  text : &'token str,
  code : Catcode
}

pub fn T_BEGIN<'token>() -> Token<'token> {
  Token { text: "{", code: Catcode::BEGIN }  
}
pub fn T_END<'token>() -> Token<'token> {
  Token { text: "}", code: Catcode::END }
}
pub fn T_MATH<'token>() -> Token<'token> {
  Token { text: "$", code: Catcode::MATH }
}
pub fn T_ALIGN<'token>() -> Token<'token> {
  Token { text: "&", code: Catcode::ALIGN }
}
pub fn T_PARAM<'token>() -> Token<'token> {
  Token { text: "#", code: Catcode::PARAM }  
}
pub fn T_SUPER<'token>() -> Token<'token> {
 Token { text: "^", code: Catcode::SUPER }
}
pub fn T_SUB<'token>() -> Token<'token> {
  Token { text: "_", code: Catcode::SUB }
}
pub fn T_SPACE<'token>() -> Token<'token> {
  Token { text: " ", code: Catcode::SPACE }
}
pub fn T_CR<'token>() -> Token<'token> {
  Token { text: "\n", code: Catcode::SPACE }
}
pub fn T_LETTER<'token>(text : &'token str) -> Token<'token> {
  Token { text : text, code: Catcode::LETTER }
}
pub fn T_CS<'token>(text : &'token str) -> Token<'token> {
  Token { text : text, code: Catcode::CS}
}
pub fn untex(digested : Digested) -> String {
  digested.to_string()
}