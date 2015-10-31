use core::Digested;

#[derive(PartialEq, Clone, Copy, Hash)]
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

#[derive(Clone, Hash)]
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
  Token { text : text.to_string(), code: Catcode::LETTER }
}
pub fn T_CS(text : String) -> Token {
  Token { text : text.to_string(), code: Catcode::CS}
}
pub fn untex(digested : Digested) -> String {
  digested.to_string()
}