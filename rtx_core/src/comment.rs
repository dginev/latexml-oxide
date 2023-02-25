use std::borrow::Cow;
use std::collections::HashMap;
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
use crate::BoxOps;

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
  fn get_properties(&self) -> &HashMap<String, Stored> {
    unimplemented!();
  }
  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    unimplemented!();
  }
  fn get_string(&self, state: &State) -> Result<Cow<str>> { Ok(Cow::Borrowed("")) }
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> {
    document.insert_comment(&self.0, state)?;
    Ok(())
  }
  fn get_font(&self, _: &mut State) -> Result<Option<Cow<Font>>> { Ok(None) }
  fn get_property(&self, key: &str) -> Option<Cow<Stored>> { None }
  fn has_property(&self, key: &str) -> bool { false }
  fn get_property_bool(&self, key: &str) -> bool { false }
  fn get_width(&mut self, options: Option<HashMap<String, Stored>>, state: &mut State) -> Result<Option<RegisterValue>> {
    Ok(Some(RegisterValue::Dimension(Dimension::new(0))))
  }

  fn compute_size(&self, options: HashMap<String, Stored>, state: &mut State) -> Result<(Dimension, Dimension, Dimension)> {
    Ok((Dimension::default(), Dimension::default(), Dimension::default()))
  }

  // sub getHeight      { return Dimension(0); }
  // sub getTotalHeight { return Dimension(0); }
  // sub getDepth       { return Dimension(0); }
  // sub getSize { return (Dimension(0), Dimension(0), Dimension(0), Dimension(0), Dimension(0), Dimension(0)); }
}
