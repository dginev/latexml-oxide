use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::numeric_ops::NumericOps;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::register::RegisterValue;
use crate::document::Document;
use crate::state::State;
use crate::tokens::Tokens;
use crate::{BoxOps, Digested};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Comment(pub String);

impl fmt::Display for Comment {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "") }
}
impl Object for Comment {
  fn get_locator(&self) -> Option<Cow<Locator>> { None }
  fn revert(&self, _state: &mut State) -> Result<Tokens> { Ok(Tokens!()) }
}
impl BoxOps for Comment {
  fn unlist(&self) -> Vec<Digested> { Vec::new() }
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored> {
    unimplemented!();
  }
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> {
    document.insert_comment(&self.0, state)?;
    Ok(())
  }
  fn get_font(&self) -> Option<Cow<Font>> { None }
  fn get_property(&self, key: &str, state: &mut State) -> Option<Cow<Stored>> { None }
  fn get_property_bool(&self, key: &str) -> bool { false }
  fn get_width(&self, state: &mut State) -> Option<RegisterValue> { Some(RegisterValue::Dimension(Dimension::new(0))) }

  // sub getHeight      { return Dimension(0); }
  // sub getTotalHeight { return Dimension(0); }
  // sub getDepth       { return Dimension(0); }
  // sub getSize { return (Dimension(0), Dimension(0), Dimension(0), Dimension(0), Dimension(0), Dimension(0)); }
}
