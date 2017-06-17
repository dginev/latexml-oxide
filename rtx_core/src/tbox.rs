use common::error::*;
use common::font::Font;
use {Digested, BoxOps};
use token::Token;
use document::Document;
use state::{ObjectStore, State};
use std::collections::HashMap;

/// Box is a Rust keyword, so we use "Tbox" instead, as in "TeX Box"
#[derive(Debug, Clone, PartialEq)]
pub struct Tbox {
  // TODO
  pub text: String,
  pub font: Font,
  pub locator: String,
  pub properties: HashMap<String, String>,
  pub tokens: Vec<Token>,
}

impl Default for Tbox {
  fn default() -> Self {
    Tbox {
      text: String::new(),
      font: Font::default(),
      locator: String::new(),
      properties: HashMap::new(),
      tokens: Vec::new(),
    }
  }
}

//======================================================================
// Exported constructors

impl Tbox {
  pub fn new(string: String, font_opt: Option<Font>, locator_opt: Option<String>, tokens_opt: Vec<Token>, properties: HashMap<String, String>, state: &mut State) -> Self {

    let font = match font_opt {
      Some(f) => f,
      None => match state.lookup_font() {
        Some(state_font) => state_font,
        None => Font::default() // should never happen
      }
    };
    // let locator = $STATE->getStomach->getGullet->getLocator unless defined $locator;
    let _locator = locator_opt;

    let tokens =  if !string.is_empty() && tokens_opt.is_empty() {
      vec![T_OTHER!(string)]
    } else {
      tokens_opt
    };

    if state.lookup_bool("IN_MATH") {
      let mut box_props = properties;
      box_props.insert("mode".to_string(),"math".to_string());
      if !string.is_empty() {
        match state.lookup_value(&format!("math_token_attributes_{}",string)) {
          Some(&ObjectStore::HashStr(ref attr)) => {
            for (key,value) in attr.iter() {
              box_props.entry(key.to_string()).or_insert(value.to_string());
            }
          },
          _ => {}
        };
      }
      let specialized_font = font.specialize(&string);
      Tbox {text: string,  tokens: tokens, font: specialized_font,// $locator,
        properties: box_props,
        ..Tbox::default()
      }
    } else {
      Tbox {text: string, font: font, // $locator,
        tokens, properties: properties,
        ..Tbox::default()
      }
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
    let font = &self.font;
    let mode = match self.properties.get("mode") {
      Some(s) => s.to_owned(),
      None => "text".to_string(),
    };
    if !text.is_empty() {
      if mode == "math" {
        try!(document.insert_math_token(text, self.properties, state));//, font => $$self[1], %{ $$self[4] })
      } else {
        try!(document.open_text(text, font, state));
      }
    }
    Ok(())
  }

  fn revert(&self) -> Vec<Token> {
    self.tokens.clone()
  }

  fn get_font(&self) -> Option<&Font> {
    Some(&self.font)
  }
}
