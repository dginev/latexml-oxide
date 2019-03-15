use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::document::Document;
use crate::state::State;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::{BoxOps, Digested};

/// Box is a Rust keyword, so we use "Tbox" instead, as in "TeX Box"
#[derive(Debug, Clone, PartialEq)]
pub struct Tbox {
  pub text: String,
  pub font: Rc<Font>,
  pub locator: Locator,
  pub properties: HashMap<String, Stored>,
  pub tokens: Tokens,
}

impl Default for Tbox {
  fn default() -> Self {
    Tbox {
      text: String::new(),
      font: Rc::new(Font::text_default()),
      locator: Locator::default(),
      properties: HashMap::new(),
      tokens: Tokens!(),
    }
  }
}

//======================================================================
// Exported constructors
impl fmt::Display for Tbox {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.text) }
}
impl Object for Tbox {
  fn get_locator(&self) -> Cow<Locator> { Cow::Borrowed(&self.locator) }
  fn revert(&self) -> Result<Tokens> { Ok(self.tokens.clone()) }
}
impl Tbox {
  pub fn new(
    text: String,
    font_opt: Option<Rc<Font>>,
    locator_opt: Option<Locator>,
    tokens_opt: Tokens,
    mut properties: HashMap<String, Stored>,
    state: &mut State,
  ) -> Self
  {
    let font = match font_opt {
      Some(f) => f,
      None => match state.lookup_font() {
        Some(state_font) => Rc::clone(&state_font),
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
      properties.insert(s!("mode"), String::from("math").into());
      if !text.is_empty() {
        if let Some(&Stored::HashString(ref attr)) = state.lookup_value(&s!("math_token_attributes_{}", text)) {
          for (key, value) in attr.iter() {
            properties.entry(key.to_string()).or_insert_with(|| Stored::String(value.to_owned()));
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
  fn unlist(&self) -> Vec<Digested> { Vec::new() }
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored> { &mut self.properties }

  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> {
    let text = &self.text;
    let font = &self.font;
    let mode: String = match self.properties.get("mode") {
      Some(Stored::String(s)) => s.to_owned(),
      _ => String::from("text"),
    };

    if !text.is_empty() {
      if mode == "math" {
        document.insert_math_token(text, Stored::cast_to_string_hash(&self.properties), Some(&self.font), state)?;
      } else {
        document.open_text(text, font, state)?;
      }
    }
    Ok(())
  }

  fn get_font(&self) -> Option<Cow<Font>> { Some(Cow::Borrowed(&self.font)) }

  fn get_property(&self, key: &str, state: &mut State) -> Option<Cow<Stored>> {
    if key == "isSpace" {
      match self.properties.get(key) {
        Some(value) => Some(Cow::Borrowed(value)),
        None => {
          let tex = self.tokens.untex(state); // !
          let property_bool = !tex.is_empty() && tex.chars().all(char::is_whitespace); // Check the TeX code, not (just) the string!
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
  fn from(tbox: Tbox) -> Option<Rc<RefCell<Digested>>> { Some(Rc::new(RefCell::new(Digested::TBox(Rc::new(tbox))))) }
}
