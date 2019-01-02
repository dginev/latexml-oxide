use std::borrow::Cow;
use std::collections::HashMap;
// use std::fmt;
// use std::rc::Rc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::store::Stored;
// use crate::definition::expandable::Expandable;
// use crate::definition::Definition;
use crate::document::Document;
// use crate::list::List;
use crate::state::State;
use crate::token::Token;
use crate::tokens::Tokens;
use crate::{BoxOps, Digested};

#[derive(Debug, Clone)]
pub struct KeyVal {
  // which KeyVals are we parsing and how do we behave?
  prefix: String,
  key: String,
  keyset: String,
}

impl Default for KeyVal {
  fn default() -> Self {
    KeyVal {
      prefix: "KV".to_string(),
      keyset: String::new(),
      key: String::new(),
    }
  }
}

impl KeyVal {
  pub fn new(prefix: Option<String>, keyset: String, key: String) -> Self {
    let prefix = prefix.unwrap_or_else(|| "KV".to_string());
    KeyVal { prefix, key, keyset }
  }

  pub fn get_header(&self) -> String { s!("{}@{}@{}", self.prefix, self.keyset, self.key) }

  //======================================================================
  // Property access
  //======================================================================

  pub fn get_prop<'a>(&self, key: &str, state: &'a State) -> Option<&'a Stored> { state.lookup_value(&s!("KEYVAL@{}@{}", key, self.get_header())) }
  pub fn get_default(&self, state: &State) -> Option<Stored> {
    match self.get_prop("default", state) {
      None => None,
      Some(v) => Some((*v).clone()),
    }
  }
}
