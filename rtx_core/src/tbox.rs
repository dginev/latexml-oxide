use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use common::error::*;
use common::font::Font;
use common::store::Stored;
use document::Document;
use state::State;
use token::{Catcode, Token};
use tokens::Tokens;
use {BoxOps, Digested};

/// Box is a Rust keyword, so we use "Tbox" instead, as in "TeX Box"
#[derive(Debug, Clone, PartialEq)]
pub struct Tbox {
  // TODO
  pub text: String,
  pub font: Rc<Font>,
  pub locator: String,
  pub properties: HashMap<String, Stored>,
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
    mut properties: HashMap<String, Stored>,
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
      Tokens!(T_OTHER!(text.clone()))
    } else {
      tokens_opt
    };

    if state.lookup_bool("IN_MATH") {
      properties.insert(s!("mode"), String::from("math").into());
      if !text.is_empty() {
        if let Some(&Stored::HashStr(ref attr)) =
          state.lookup_value(&s!("math_token_attributes_{}", text))
        {
          for (key, value) in attr.iter() {
            properties
              .entry(key.to_string())
              .or_insert_with(|| Stored::String(value.to_owned()));
          }
        }
      }
      let font = Rc::new(font.specialize(&text));
      Tbox {
        text,
        font, // $locator,
        tokens,
        properties,
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
  fn unlist(&self) -> Vec<Digested> { Vec::new() }

  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> {
    let text = &self.text;
    let font = &self.font;
    let mode: String = match self.properties.get("mode") {
      Some(Stored::String(s)) => s.to_owned(),
      _ => String::from("text"),
    };

    if !text.is_empty() {
      if mode == "math" {
        document.insert_math_token(
          text,
          Stored::to_string_hash(&self.properties),
          Some(&self.font),
          state,
        )?;
      } else {
        document.open_text(text, font, state)?;
      }
    }
    Ok(())
  }

  fn revert(&self) -> Tokens { self.tokens.clone() }

  fn get_font(&self) -> Option<Cow<Font>> { Some(Cow::Borrowed(&self.font)) }

  fn get_property(&self, key: &str, state: &mut State) -> Option<Cow<Stored>> {
    if key == "isSpace" {
      match self.properties.get(key) {
        Some(value) => Some(Cow::Borrowed(value)),
        None => {
          let tex = self.tokens.untex(state); // !
          let property_bool = !tex.is_empty() && tex.chars().all(|c| c.is_whitespace()); // Check the TeX code, not (just) the string!
          Some(Cow::Borrowed(property_bool.into()))
        },
      }
    } else {
      match self.properties.get(key) {
        None => None,
        Some(v) => Some(Cow::Borrowed(v)),
      }
    }
  }
}

impl From<Tbox> for Result<Vec<Digested>> {
  fn from(tbox: Tbox) -> Result<Vec<Digested>> { Ok(vec![Digested::TBox(Rc::new(tbox))]) }
}
impl From<Tbox> for Option<Digested> {
  fn from(tbox: Tbox) -> Option<Digested> { Some(Digested::TBox(Rc::new(tbox))) }
}
impl From<Tbox> for Option<Rc<RefCell<Digested>>> {
  fn from(tbox: Tbox) -> Option<Rc<RefCell<Digested>>> {
    Some(Rc::new(RefCell::new(Digested::TBox(Rc::new(tbox)))))
  }
}
