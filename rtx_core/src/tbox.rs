use std::collections::HashMap;
use std::rc::Rc;

use common::error::*;
use common::font::Font;
use document::Document;
use state::{ObjectStore, State};
use tokens::Tokens;
use {BoxOps, Digested};

/// Box is a Rust keyword, so we use "Tbox" instead, as in "TeX Box"
#[derive(Debug, Clone, PartialEq)]
pub struct Tbox {
  // TODO
  pub text: String,
  pub font: Rc<Font>,
  pub locator: String,
  pub properties: HashMap<String, String>,
  pub tokens: Tokens,
}

impl Default for Tbox {
  fn default() -> Self {
    Tbox {
      text: String::new(),
      font: Rc::new(Font::text_default()),
      locator: String::new(),
      properties: HashMap::new(),
      tokens: Tokens!(),
    }
  }
}

//======================================================================
// Exported constructors

impl Tbox {
  pub fn new(
    text: String,
    font_opt: Option<Rc<Font>>,
    locator_opt: Option<String>,
    tokens_opt: Tokens,
    properties: HashMap<String, String>,
    state: &mut State,
  ) -> Self
  {
    let font = match font_opt {
      Some(f) => f,
      None => match state.lookup_font() {
        Some(state_font) => state_font.clone(),
        None => Rc::new(Font::text_default()), // should never happen
      },
    };
    // let locator = $STATE->getStomach->getGullet->getLocator unless defined $locator;
    let _locator = locator_opt;

    let tokens = if !text.is_empty() && tokens_opt.is_empty() {
      Tokens!(T_OTHER!(text))
    } else {
      tokens_opt
    };

    if state.lookup_bool("IN_MATH") {
      let mut box_props = properties;
      box_props.insert(s!("mode"), s!("math"));
      if !text.is_empty() {
        if let Some(&ObjectStore::HashStr(ref attr)) =
          state.lookup_value(&s!("math_token_attributes_{}", text))
        {
          for (key, value) in attr.iter() {
            box_props
              .entry(key.to_string())
              .or_insert_with(|| value.to_string());
          }
        }
      }
      let specialized_font = font.specialize(&text);
      Tbox {
        text,
        tokens,
        font: Rc::new(specialized_font), // $locator,
        properties: box_props,
        ..Tbox::default()
      }
    } else {
      Tbox {
        text,
        font, // $locator,
        tokens,
        properties,
        ..Tbox::default()
      }
    }
  }
}

impl BoxOps for Tbox {
  fn to_string(&self) -> String { self.text.clone() }
  fn unlist(self) -> Vec<Digested> { Vec::new() }

  fn be_absorbed(self, document: &mut Document, state: &mut State) -> Result<()> {
    let text = &self.text;
    let font = &self.font;
    let mode = match self.properties.get("mode") {
      Some(s) => s.to_owned(),
      None => s!("text"),
    };
    if !text.is_empty() {
      if mode == "math" {
        document.insert_math_token(text, self.properties, Some(&self.font), state)?;
      } else {
        document.open_text(text, font, state)?;
      }
    }
    Ok(())
  }

  fn revert(&self) -> Tokens { self.tokens.clone() }

  fn get_font(&self) -> Option<&Font> { Some(&self.font) }
}
