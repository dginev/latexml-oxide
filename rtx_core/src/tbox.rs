use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

use crate::common::dimension::Dimension;
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
#[derive(Debug, Clone)]
pub struct Tbox {
  /// plain-text content
  pub text: String,
  /// associated font for `text`
  pub font: Arc<Font>,
  /// source location where the box originated
  pub locator: Locator,
  /// misc properties, such as sizing information
  pub properties: HashMap<String, Stored>,
  /// a Tokens list containing the TeX that created (or could have) the Tbox.
  pub tokens: Tokens,
}

impl Default for Tbox {
  fn default() -> Self {
    Tbox {
      text: String::new(),
      font: Arc::new(Font::text_default()),
      locator: Locator::default(),
      properties: HashMap::default(),
      tokens: Tokens!(),
    }
  }
}

impl PartialEq for Tbox {
  // Should this compare fonts too?
  fn eq(&self, other: &Self) -> bool { self.text == other.text && *self.font == *other.font }
}

//======================================================================
// Exported constructors
impl fmt::Display for Tbox {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.text) }
}
impl Object for Tbox {
  fn get_locator(&self) -> Option<Cow<Locator>> { Some(Cow::Borrowed(&self.locator)) }
  fn revert(&self, _state: &State) -> Result<Tokens> { Ok(self.tokens.clone()) }
  fn stringify(&self) -> String { format!("{self:?}") }
}
impl Tbox {
  /// creates a new Tbox.
  /// If `font_opt` or `locator_opt` are None, they are obtained from the
  /// currently active State.  Note that `text` can
  /// be empty, which contributes nothing to the generated document,
  /// but does record the TeX code (in the tokens).
  pub fn new(
    text: String,
    font_opt: Option<Arc<Font>>,
    locator_opt: Option<Locator>,
    tokens_opt: Tokens,
    mut properties: HashMap<String, Stored>,
    state: &mut State,
  ) -> Self {
    let font = match font_opt {
      Some(f) => f,
      None => state.lookup_font().unwrap(),
    };
    // let locator = $STATE->getStomach->getGullet->getLocator unless defined $locator;
    let _locator = locator_opt;

    let tokens = if !text.is_empty() && tokens_opt.is_empty() {
      Tokens!(T_OTHER!(&text))
    } else {
      tokens_opt
    };

    if properties.contains_key("isSpace")
      && (properties.contains_key("width")
        || properties.contains_key("height")
        || properties.contains_key("depth"))
    {
      properties
        .entry(String::from("width"))
        .or_insert_with(|| Stored::Dimension(Dimension::default()));
      properties
        .entry("height".to_string())
        .or_insert_with(|| Stored::Dimension(Dimension::default()));
      properties
        .entry("depth".to_string())
        .or_insert_with(|| Stored::Dimension(Dimension::default()));
    }
    if state.lookup_bool("IN_MATH") {
      properties.insert(s!("mode"), String::from("math").into());
      if !text.is_empty() {
        if let Some(Stored::HashString(attr)) =
          state.lookup_value(&s!("math_token_attributes_{}", text))
        {
          for (key, value) in attr.iter() {
            properties
              .entry(key.to_string())
              .or_insert_with(|| Stored::String(value.to_owned()));
          }
        }
      }
      let font = Arc::new(font.specialize(&text));
      Tbox {
        text,
        font, // $locator,
        properties,
        tokens,
        ..Tbox::default()
      }
    } else {
      Tbox {
        text,
        font, // $locator,
        properties,
        tokens,
        ..Tbox::default()
      }
    }
  }
  /// checks if the text content is empty
  pub fn is_empty(&self) -> bool { self.text.is_empty() }
}

impl BoxOps for Tbox {
  fn get_tokens(&self) -> Option<&Tokens> { Some(&self.tokens) }
  fn get_properties(&self) -> &HashMap<String, Stored> { &self.properties }
  fn get_property_bool(&self, key: &str) -> bool {
    matches!(self.properties.get(key), Some(Stored::Bool(true)))
  }
  fn has_property(&self, key: &str) -> bool { self.properties.contains_key(key) }
  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    self.properties.insert(key.to_string(), value.into());
  }
  fn get_string(&self, _state: &State) -> Result<Cow<'_, str>> {
    Ok(Cow::Borrowed(self.text.as_str()))
  }

  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<Vec<Node>> {
    let text = &self.text;
    let font = &self.font;
    let mode = match self.properties.get("mode") {
      Some(Stored::String(s)) => s.as_str(),
      _ => "text",
    };

    if !text.is_empty() {
      if mode == "math" {
        Ok(vec![document.insert_math_token(
          text,
          Stored::cast_to_string_hash(&self.properties),
          Some(font),
          state,
        )?])
      } else {
        match document.open_text(text, font, state)? {
          None => Ok(Vec::new()),
          Some(node) => Ok(vec![node]),
        }
      }
    } else {
      Ok(Vec::new())
    }
  }

  fn get_font(&self, _: &mut State) -> Result<Option<Cow<Font>>> {
    Ok(Some(Cow::Borrowed(&self.font)))
  }

  fn compute_size(
    &self,
    options: HashMap<String, Stored>,
    state: &mut State,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    if let Some(body_stored) = self.get_property("body") {
      if let Stored::Digested(ref body) = *body_stored {
        body.compute_size(options, state)
      } else {
        panic!("the stored 'body' property should always be a Stored::Digested enum case.");
      }
    } else {
      Ok(
        self
          .font
          .compute_string_size(&self.get_string(state)?, options, state),
      )
    }
  }
}

impl From<Tbox> for Result<Vec<Digested>> {
  fn from(tbox: Tbox) -> Result<Vec<Digested>> { Ok(vec![Digested::from(tbox)]) }
}
impl From<Tbox> for Option<Digested> {
  fn from(tbox: Tbox) -> Option<Digested> { Some(Digested::from(tbox)) }
}
