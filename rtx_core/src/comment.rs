use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::fmt;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::register::RegisterValue;
use crate::document::Document;
use crate::state::State;
use crate::tokens::Tokens;
use crate::tokens::NO_TOKENS;
use crate::{BoxOps, NO_PROPERTIES};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Comment(pub String);

impl fmt::Display for Comment {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "") }
}
impl Object for Comment {
  fn get_locator(&self) -> Option<Cow<Locator>> { None }
  fn revert(&self, _state: &State) -> Result<Tokens> { Ok(NO_TOKENS) }
}
impl BoxOps for Comment {
  fn with_properties<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&HashMap<String, Stored>) -> R {
    caller(&NO_PROPERTIES)
  }
  fn set_property<T: Into<Stored>>(&mut self, _key: &str, _value: T) {} // no-op
  fn get_string(&self, _state: &State) -> Result<Cow<str>> { Ok(Cow::Borrowed(&self.0)) }
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<Vec<Node>> {
    document.insert_comment(&self.0, state)?;
    Ok(Vec::new())
  }
  fn get_font(&self, _: &mut State) -> Result<Option<Cow<Font>>> { Ok(None) }
  fn get_width(
    &self,
    _options: Option<HashMap<String, Stored>>,
    _state: &mut State,
  ) -> Result<Option<RegisterValue>> {
    Ok(Some(RegisterValue::Dimension(Dimension::new(0))))
  }

  fn compute_size(
    &self,
    _options: HashMap<String, Stored>,
    _state: &mut State,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    Ok((
      Dimension::default(),
      Dimension::default(),
      Dimension::default(),
    ))
  }

  // sub getHeight      { return Dimension(0); }
  // sub getTotalHeight { return Dimension(0); }
  // sub getDepth       { return Dimension(0); }
  // sub getSize { return (Dimension(0), Dimension(0), Dimension(0), Dimension(0), Dimension(0),
  // Dimension(0)); }
}
