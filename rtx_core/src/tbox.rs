use common::error::*;
use {Digested, BoxOps};
use token::Token;
use document::Document;
use state::State;
use std::collections::HashMap;

/// Box is a Rust keyword, so we use "Tbox" instead, as in "TeX Box"
#[derive(Debug, Clone, PartialEq)]
pub struct Tbox {
  // TODO
  pub text: String,
  pub font: String,
  pub locator: String,
  pub properties: HashMap<String, String>,
  pub tokens: Vec<Token>,
}

impl Default for Tbox {
  fn default() -> Self {
    Tbox {
      text: String::new(),
      font: String::new(),
      locator: String::new(),
      properties: HashMap::new(),
      tokens: Vec::new(),
    }
  }
}

impl BoxOps for Tbox {
  fn to_string(&self) -> String {
    self.text.clone()
  }
  fn unlist(self) -> Vec<Digested> {
    Vec::new()
  }

  fn be_absorbed(self, document: &mut Document, state: &mut State) -> Result<()> {
    let text = &self.text;
    let mode = match self.properties.get("mode") {
      Some(s) => s.to_owned(),
      None => "text".to_string(),
    };
    if !text.is_empty() {
      if mode == "math" {
        try!(document.insert_math_token(text, self.properties, state));//, font => $$self[1], %{ $$self[4] })
      } else {
        try!(document.open_text(text, state));//, $$self[1]))
      }
    }
    Ok(())
  }

  fn revert(&self) -> Vec<Token> {
    self.tokens.clone()
  }
}
